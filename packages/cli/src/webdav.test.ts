import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { renderMultistatus, resolveDavPath } from "./webdav";

const tempRoots: string[] = [];

async function makeTempDir(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "nebula-webdav-"));
  tempRoots.push(root);
  return root;
}

afterEach(async () => {
  await Promise.all(tempRoots.splice(0).map((root) => fs.rm(root, { force: true, recursive: true })));
});

describe("resolveDavPath", () => {
  it("rejects lexical traversal outside the document root", async () => {
    const root = await makeTempDir();
    await expect(resolveDavPath(root, "/../outside.txt")).resolves.toBeNull();
  });

  it("rejects symlink traversal after resolving the real path", async () => {
    const root = await makeTempDir();
    const outside = await makeTempDir();
    const outsideFile = path.join(outside, "secret.txt");
    await fs.writeFile(outsideFile, "secret");

    const linkPath = path.join(root, "secret.txt");
    try {
      await fs.symlink(outsideFile, linkPath, "file");
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "EPERM") {
        return;
      }
      throw error;
    }

    await expect(resolveDavPath(root, "/secret.txt")).resolves.toBeNull();
  });
});

describe("renderMultistatus", () => {
  it("escapes XML-sensitive href content", async () => {
    const root = await makeTempDir();
    const filePath = path.join(root, "a&b.txt");
    await fs.writeFile(filePath, "content");
    const stat = await fs.stat(filePath);

    const body = renderMultistatus(root, [{ filePath, stat }]);

    expect(body).toContain("/a&amp;b.txt");
    expect(body).not.toContain("/a&b.txt");
  });
});
