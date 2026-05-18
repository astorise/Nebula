#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const INPUT_TOPIC: &str = "nebula.fs.file_updated";
pub const OUTPUT_TOPIC: &str = "nebula.doc.markdown_ready";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileUpdatedEvent {
    pub path: String,
    pub mime_type: String,
    pub sha256: String,
    pub tunnel_host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkdownReadyEvent {
    pub source_path: String,
    pub source_sha256: String,
    pub markdown: String,
    pub markdown_sha256: String,
}

pub trait WebDavClient {
    fn fetch(&mut self, tunnel_host: &str, path: &str) -> Result<Vec<u8>>;
}

pub trait EventBus {
    fn publish(&mut self, topic: &str, event: &MarkdownReadyEvent) -> Result<()>;
}

pub fn ingest_file(
    client: &mut impl WebDavClient,
    bus: &mut impl EventBus,
    event: FileUpdatedEvent,
) -> Result<MarkdownReadyEvent> {
    let bytes = client.fetch(&event.tunnel_host, &event.path)?;
    let markdown = parse_document_to_markdown(&bytes, &event.mime_type)?;
    let output = MarkdownReadyEvent {
        source_path: event.path,
        source_sha256: event.sha256,
        markdown_sha256: sha256_hex(markdown.as_bytes()),
        markdown,
    };

    bus.publish(OUTPUT_TOPIC, &output)?;
    Ok(output)
}

pub fn parse_document_to_markdown(bytes: &[u8], mime_type: &str) -> Result<String> {
    match mime_type {
        "text/markdown" => String::from_utf8(bytes.to_vec()).map_err(Into::into),
        "text/plain" | "application/pdf" => deterministic_text_to_markdown(bytes),
        other => Err(anyhow!("unsupported document mime type: {other}")),
    }
}

fn deterministic_text_to_markdown(bytes: &[u8]) -> Result<String> {
    let text = String::from_utf8_lossy(bytes);
    let mut markdown = String::new();
    let mut previous_blank = true;

    for line in text.lines().map(str::trim) {
        if line.is_empty() {
            if !previous_blank {
                markdown.push('\n');
            }
            previous_blank = true;
            continue;
        }

        if looks_like_heading(line) {
            markdown.push_str("# ");
            markdown.push_str(line);
            markdown.push_str("\n\n");
        } else {
            markdown.push_str(line);
            markdown.push_str("\n\n");
        }
        previous_blank = false;
    }

    let normalized = markdown.trim();
    if normalized.is_empty() {
        return Err(anyhow!("document did not contain extractable text"));
    }

    Ok(format!("{normalized}\n"))
}

fn looks_like_heading(line: &str) -> bool {
    let letters = line.chars().filter(|ch| ch.is_alphabetic()).count();
    letters > 3 && line.len() <= 96 && line == line.to_uppercase()
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Client(Vec<u8>);
    struct Bus(usize);

    impl WebDavClient for Client {
        fn fetch(&mut self, _tunnel_host: &str, _path: &str) -> Result<Vec<u8>> {
            Ok(self.0.clone())
        }
    }

    impl EventBus for Bus {
        fn publish(&mut self, _topic: &str, _event: &MarkdownReadyEvent) -> Result<()> {
            self.0 += 1;
            Ok(())
        }
    }

    #[test]
    // spec: doc-parser
    fn converts_plain_text_to_markdown() {
        let mut client = Client(b"ARCHITECTURE\n\nBody paragraph".to_vec());
        let mut bus = Bus(0);
        let output = ingest_file(
            &mut client,
            &mut bus,
            FileUpdatedEvent {
                path: "/docs/a.txt".into(),
                mime_type: "text/plain".into(),
                sha256: "source".into(),
                tunnel_host: "https://webdav.tenant-acme.wormhole.internal".into(),
            },
        )
        .unwrap();

        assert!(output.markdown.starts_with("# ARCHITECTURE"));
        assert_eq!(bus.0, 1);
    }
}
