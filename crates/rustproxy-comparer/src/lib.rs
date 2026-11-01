use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffTag {
    Equal,
    Insert,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub content: String,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub additions: usize,
    pub deletions: usize,
    pub similarity_ratio: f32,
}

pub struct Comparer;

impl Comparer {
    pub fn diff_text(old: &str, new: &str) -> DiffResult {
        let diff = TextDiff::from_lines(old, new);
        let mut lines = Vec::new();
        let mut additions = 0;
        let mut deletions = 0;

        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            let _ = idx;
            for op in group {
                for change in diff.iter_changes(op) {
                    let (tag, old_l, new_l) = match change.tag() {
                        ChangeTag::Equal => {
                            (DiffTag::Equal, change.old_index(), change.new_index())
                        }
                        ChangeTag::Delete => {
                            deletions += 1;
                            (DiffTag::Delete, change.old_index(), None)
                        }
                        ChangeTag::Insert => {
                            additions += 1;
                            (DiffTag::Insert, None, change.new_index())
                        }
                    };

                    lines.push(DiffLine {
                        tag,
                        content: change.value().to_string(),
                        old_line: old_l,
                        new_line: new_l,
                    });
                }
            }
        }

        let similarity_ratio = diff.ratio();

        DiffResult {
            lines,
            additions,
            deletions,
            similarity_ratio,
        }
    }

    pub fn diff_bytes(old: &[u8], new: &[u8]) -> DiffResult {
        let old_hex = hex::encode_upper(old);
        let new_hex = hex::encode_upper(new);
        Self::diff_text(&old_hex, &new_hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparer_diff() {
        let text1 = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"auth\": true}";
        let text2 =
            "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\n\r\n{\"auth\": false}";

        let result = Comparer::diff_text(text1, text2);
        assert!(result.additions > 0);
        assert!(result.deletions > 0);
        assert!(result.similarity_ratio > 0.0 && result.similarity_ratio < 1.0);
    }
}
