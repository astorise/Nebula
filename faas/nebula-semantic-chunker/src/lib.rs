#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const INPUT_TOPIC: &str = "nebula.doc.markdown_ready";
pub const DEFAULT_EMBEDDING_MODEL: &str = "all-MiniLM-L6-v2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkdownReadyEvent {
    pub source_path: String,
    pub source_sha256: String,
    pub markdown: String,
    pub markdown_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticChunk {
    pub text: String,
    pub heading_path: Vec<String>,
    pub source_path: String,
    pub source_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorRecord {
    pub chunk: SemanticChunk,
    pub embedding: Vec<f32>,
}

pub trait InferenceHost {
    fn embed(&mut self, model: &str, text: &str) -> Result<Vec<f32>>;
}

pub trait VectorStore {
    fn upsert(&mut self, record: &VectorRecord) -> Result<()>;
}

pub fn chunk_embed_and_store(
    inference: &mut impl InferenceHost,
    store: &mut impl VectorStore,
    event: MarkdownReadyEvent,
) -> Result<Vec<VectorRecord>> {
    let chunks = split_markdown(&event)?;
    let mut records = Vec::with_capacity(chunks.len());

    for chunk in chunks {
        let embedding = inference.embed(DEFAULT_EMBEDDING_MODEL, &chunk.text)?;
        let record = VectorRecord { chunk, embedding };
        store.upsert(&record)?;
        records.push(record);
    }

    Ok(records)
}

pub fn split_markdown(event: &MarkdownReadyEvent) -> Result<Vec<SemanticChunk>> {
    let mut heading_stack: Vec<(usize, String)> = Vec::new();
    let mut current_body = Vec::new();
    let mut current_headings = Vec::new();
    let mut chunks = Vec::new();

    for line in event.markdown.lines() {
        if let Some((level, title)) = parse_heading(line) {
            flush_chunk(event, &mut chunks, &current_headings, &mut current_body);
            heading_stack.retain(|(existing_level, _)| *existing_level < level);
            heading_stack.push((level, title));
            current_headings = heading_stack
                .iter()
                .map(|(_, value)| value.clone())
                .collect();
        } else if !line.trim().is_empty() {
            current_body.push(line.trim().to_string());
        }
    }

    flush_chunk(event, &mut chunks, &current_headings, &mut current_body);

    if chunks.is_empty() {
        return Err(anyhow!("markdown did not produce semantic chunks"));
    }

    Ok(chunks)
}

fn flush_chunk(
    event: &MarkdownReadyEvent,
    chunks: &mut Vec<SemanticChunk>,
    headings: &[String],
    body: &mut Vec<String>,
) {
    if body.is_empty() {
        return;
    }

    let mut text = String::new();
    if !headings.is_empty() {
        text.push_str(&headings.join(" > "));
        text.push_str("\n\n");
    }
    text.push_str(&body.join("\n"));

    chunks.push(SemanticChunk {
        text,
        heading_path: headings.to_vec(),
        source_path: event.source_path.clone(),
        source_sha256: event.source_sha256.clone(),
    });
    body.clear();
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 || !trimmed[level..].starts_with(' ') {
        return None;
    }

    Some((level, trimmed[level + 1..].trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Inference;
    struct Store(usize);

    impl InferenceHost for Inference {
        fn embed(&mut self, _model: &str, text: &str) -> Result<Vec<f32>> {
            Ok(vec![text.len() as f32, 1.0])
        }
    }

    impl VectorStore for Store {
        fn upsert(&mut self, _record: &VectorRecord) -> Result<()> {
            self.0 += 1;
            Ok(())
        }
    }

    #[test]
    // spec: semantic-chunker
    fn keeps_hierarchical_heading_context() {
        let event = MarkdownReadyEvent {
            source_path: "/docs/a.md".into(),
            source_sha256: "source".into(),
            markdown_sha256: "markdown".into(),
            markdown: "# A\n## B\n### C\nParagraph".into(),
        };

        let chunks = split_markdown(&event).unwrap();
        assert_eq!(chunks[0].heading_path, vec!["A", "B", "C"]);
        assert!(chunks[0].text.starts_with("A > B > C"));
    }

    #[test]
    // spec: semantic-chunker
    fn embeds_and_persists_chunks() {
        let event = MarkdownReadyEvent {
            source_path: "/docs/a.md".into(),
            source_sha256: "source".into(),
            markdown_sha256: "markdown".into(),
            markdown: "# A\nBody".into(),
        };
        let mut inference = Inference;
        let mut store = Store(0);

        let records = chunk_embed_and_store(&mut inference, &mut store, event).unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(store.0, 1);
    }
}
