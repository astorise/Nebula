import crypto from "node:crypto";
import fs from "node:fs/promises";
import { watch, type FSWatcher } from "node:fs";
import path from "node:path";
import type { IncomingMessage, ServerResponse } from "node:http";
import type { TachyonRouter } from "./tachyon";

const READ_METHODS = new Set(["GET", "HEAD", "OPTIONS", "PROPFIND"]);
const MUTATING_METHODS = new Set(["PUT", "POST", "DELETE", "MKCOL", "MOVE", "COPY", "PATCH", "LOCK", "UNLOCK", "PROPPATCH"]);

export interface WebDavContext {
  docsRoot: string;
}

export interface FileUpdatedPayload {
  path: string;
  mime_type: string;
  sha256: string;
}

export async function handleWebDav(req: IncomingMessage, res: ServerResponse, context: WebDavContext): Promise<void> {
  const method = req.method?.toUpperCase() ?? "GET";

  if (MUTATING_METHODS.has(method) || !READ_METHODS.has(method)) {
    res.writeHead(405, {
      Allow: "GET, HEAD, OPTIONS, PROPFIND"
    });
    res.end();
    return;
  }

  if (method === "OPTIONS") {
    res.writeHead(204, {
      Allow: "GET, HEAD, OPTIONS, PROPFIND",
      DAV: "1"
    });
    res.end();
    return;
  }

  const targetPath = resolveDavPath(context.docsRoot, req.url ?? "/");
  if (!targetPath) {
    res.writeHead(403);
    res.end("Forbidden");
    return;
  }

  try {
    const stat = await fs.stat(targetPath);

    if (method === "PROPFIND") {
      await handlePropfind(req, res, context.docsRoot, targetPath, stat);
      return;
    }

    if (stat.isDirectory()) {
      res.writeHead(403);
      res.end("Directory listing is only available via PROPFIND");
      return;
    }

    const file = await fs.readFile(targetPath);
    res.writeHead(200, {
      "Content-Length": file.byteLength
    });

    if (method === "HEAD") {
      res.end();
      return;
    }

    res.end(file);
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code;
    res.writeHead(code === "ENOENT" ? 404 : 500);
    res.end(code === "ENOENT" ? "Not found" : "Internal server error");
  }
}

function resolveDavPath(root: string, requestUrl: string): string | null {
  const decodedPath = decodeURIComponent(new URL(requestUrl, "https://nebula.local").pathname);
  const resolved = path.resolve(root, `.${decodedPath}`);
  const relative = path.relative(root, resolved);

  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    return null;
  }

  return resolved;
}

async function handlePropfind(
  req: IncomingMessage,
  res: ServerResponse,
  root: string,
  targetPath: string,
  stat: Awaited<ReturnType<typeof fs.stat>>
): Promise<void> {
  const depth = req.headers.depth === "1" ? 1 : 0;
  const entries = [{ filePath: targetPath, stat }];

  if (depth === 1 && stat.isDirectory()) {
    const children = await fs.readdir(targetPath);
    for (const child of children) {
      const childPath = path.join(targetPath, child);
      entries.push({ filePath: childPath, stat: await fs.stat(childPath) });
    }
  }

  const body = renderMultistatus(root, entries);
  res.writeHead(207, {
    "Content-Type": "application/xml; charset=utf-8",
    "Content-Length": Buffer.byteLength(body)
  });
  res.end(body);
}

function renderMultistatus(root: string, entries: Array<{ filePath: string; stat: Awaited<ReturnType<typeof fs.stat>> }>): string {
  const responses = entries.map(({ filePath, stat }) => {
    const href = `/${path.relative(root, filePath).replaceAll(path.sep, "/")}`;
    const normalizedHref = href === "/" ? "/" : encodeURI(href);
    const resourceType = stat.isDirectory() ? "<D:resourcetype><D:collection /></D:resourcetype>" : "<D:resourcetype />";

    return [
      "  <D:response>",
      `    <D:href>${escapeXml(normalizedHref)}</D:href>`,
      "    <D:propstat>",
      "      <D:prop>",
      `        <D:getcontentlength>${stat.size}</D:getcontentlength>`,
      `        <D:getlastmodified>${stat.mtime.toUTCString()}</D:getlastmodified>`,
      `        ${resourceType}`,
      "      </D:prop>",
      "      <D:status>HTTP/1.1 200 OK</D:status>",
      "    </D:propstat>",
      "  </D:response>"
    ].join("\n");
  });

  return `<?xml version="1.0" encoding="utf-8"?>\n<D:multistatus xmlns:D="DAV:">\n${responses.join("\n")}\n</D:multistatus>`;
}

function escapeXml(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;").replaceAll('"', "&quot;");
}

export function watchWebDavRoot(docsRoot: string, router: TachyonRouter, debounceMs = 2000): FSWatcher {
  const timers = new Map<string, NodeJS.Timeout>();
  const watcher = watch(docsRoot, { recursive: true }, (_eventType, filename) => {
    if (!filename) {
      return;
    }

    const absolutePath = path.resolve(docsRoot, filename.toString());
    const relative = path.relative(docsRoot, absolutePath);
    if (relative.startsWith("..") || path.isAbsolute(relative)) {
      return;
    }

    const existing = timers.get(absolutePath);
    if (existing) {
      clearTimeout(existing);
    }

    timers.set(
      absolutePath,
      setTimeout(() => {
        timers.delete(absolutePath);
        void emitFileUpdated(docsRoot, absolutePath, router);
      }, debounceMs)
    );
  });

  watcher.on("close", () => {
    for (const timer of timers.values()) {
      clearTimeout(timer);
    }
    timers.clear();
  });

  return watcher;
}

async function emitFileUpdated(docsRoot: string, absolutePath: string, router: TachyonRouter): Promise<void> {
  try {
    const stat = await fs.stat(absolutePath);
    if (!stat.isFile()) {
      return;
    }

    const file = await fs.readFile(absolutePath);
    const relativePath = `/${path.relative(docsRoot, absolutePath).replaceAll(path.sep, "/")}`;
    const payload: FileUpdatedPayload = {
      path: relativePath,
      mime_type: mimeTypeFor(absolutePath),
      sha256: crypto.createHash("sha256").update(file).digest("hex")
    };

    router.emitEvent({
      type: "EVENT",
      payload: {
        topic: "nebula.fs.file_updated",
        payload
      }
    });
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code;
    if (code !== "ENOENT") {
      router.emitEvent({
        type: "nebula.fs.watch_error",
        payload: {
          path: absolutePath,
          message: error instanceof Error ? error.message : "Unknown watcher error"
        }
      });
    }
  }
}

function mimeTypeFor(filePath: string): string {
  switch (path.extname(filePath).toLowerCase()) {
    case ".pdf":
      return "application/pdf";
    case ".md":
    case ".markdown":
      return "text/markdown";
    case ".docx":
      return "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
    case ".txt":
      return "text/plain";
    default:
      return "application/octet-stream";
  }
}
