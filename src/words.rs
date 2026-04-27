//! Walk the repository, extract candidate human‑language words, and
//! report any that aren't covered by the ontology or the allow‑list.
//!
//! Used by the `meta-ontology check-words <path>` CLI command and by
//! the CI workflow step.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::Ontology;

/// One unknown word reported back to the caller.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnknownWord {
    pub word: String,
    pub file: PathBuf,
    pub line: usize,
}

/// Default repo‑relative directories the scanner skips entirely.
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".github",
    "target",
    "node_modules",
    "data",
    "scripts",
    "experiments",
    "examples",
    "changelog.d",
    "tests",
    "ci-logs",
];

/// File extensions whose prose content the scanner reads.
const SCAN_EXTS: &[&str] = &["md"];

/// Scan a path (file or directory) for words not covered by `ontology`
/// or `allowlist.lino`.
pub fn scan_path(root: &Path, ontology: &Ontology) -> Result<Vec<UnknownWord>, std::io::Error> {
    let mut unknown = BTreeSet::new();
    if root.is_file() {
        scan_file(root, ontology, &mut unknown)?;
        return Ok(unknown.into_iter().collect());
    }
    for entry in WalkDir::new(root).into_iter().filter_entry(|e| {
        // Skip listed directories and any dotfile dirs (.git, .github, etc.)
        if e.file_type().is_dir() {
            let name = e.file_name().to_string_lossy();
            if name.starts_with('.') && e.depth() > 0 {
                return false;
            }
            !SKIP_DIRS.contains(&name.as_ref())
        } else {
            true
        }
    }) {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if !SCAN_EXTS.contains(&ext) {
            continue;
        }
        scan_file(entry.path(), ontology, &mut unknown)?;
    }
    Ok(unknown.into_iter().collect())
}

fn scan_file(
    path: &Path,
    ontology: &Ontology,
    unknown: &mut BTreeSet<UnknownWord>,
) -> Result<(), std::io::Error> {
    let text = std::fs::read_to_string(path)?;
    let mut in_code_block = false;
    for (lineno, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        let stripped = strip_inline_noise(line);
        for word in tokenize(&stripped) {
            let normalized = normalize(&word);
            if normalized.is_empty() {
                continue;
            }
            // Skip single-letter words — they're invariably milestone IDs
            // (M3, M4), section numbers, list bullets in disguise, etc.
            if normalized.chars().count() <= 1 {
                continue;
            }
            if ontology.covers(&normalized) {
                continue;
            }
            // also try stripped trailing 's' / 'es' / 'ed' / 'ing' for trivial
            // English plural / past forms
            if try_lemmas(&normalized).any(|lemma| ontology.covers(&lemma)) {
                continue;
            }
            unknown.insert(UnknownWord {
                word: normalized,
                file: path.to_path_buf(),
                line: lineno + 1,
            });
        }
    }
    Ok(())
}

/// Remove inline code spans, image refs, links (text + url), HTML
/// tags, and bare URLs from a Markdown line so the tokenizer doesn't
/// see identifier noise.
fn strip_inline_noise(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_code = false;
    while let Some(c) = chars.next() {
        match c {
            '`' => {
                in_code = !in_code;
                out.push(' ');
            }
            _ if in_code => out.push(' '),
            '<' => {
                // skip to '>'
                for c2 in chars.by_ref() {
                    if c2 == '>' {
                        break;
                    }
                }
                out.push(' ');
            }
            '!' if chars.peek() == Some(&'[') => {
                // image: ![alt](url) — drop alt and url entirely
                chars.next(); // consume '['
                let mut depth = 1;
                for c2 in chars.by_ref() {
                    if c2 == '[' {
                        depth += 1;
                    } else if c2 == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut pdepth = 1;
                    for c2 in chars.by_ref() {
                        if c2 == '(' {
                            pdepth += 1;
                        } else if c2 == ')' {
                            pdepth -= 1;
                            if pdepth == 0 {
                                break;
                            }
                        }
                    }
                }
                out.push(' ');
            }
            '[' => {
                // [text](url) — keep text, drop url
                let mut depth = 1;
                for c2 in chars.by_ref() {
                    if c2 == '[' {
                        depth += 1;
                    } else if c2 == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    } else {
                        out.push(c2);
                    }
                }
                if chars.peek() == Some(&'(') {
                    chars.next();
                    let mut pdepth = 1;
                    for c2 in chars.by_ref() {
                        if c2 == '(' {
                            pdepth += 1;
                        } else if c2 == ')' {
                            pdepth -= 1;
                            if pdepth == 0 {
                                break;
                            }
                        }
                    }
                }
                out.push(' ');
            }
            _ => out.push(c),
        }
    }
    // Strip bare URLs (http(s)://… and www.…) by replacing whole
    // tokens that look like URLs with spaces.
    let mut cleaned = String::with_capacity(out.len());
    for token in out.split_whitespace() {
        let stripped = token.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '/');
        if !looks_like_url_or_path(stripped) {
            cleaned.push_str(token);
        }
        cleaned.push(' ');
    }
    cleaned
}

fn looks_like_url_or_path(token: &str) -> bool {
    if token.starts_with("http://")
        || token.starts_with("https://")
        || token.starts_with("www.")
        || token.starts_with("./")
        || token.starts_with("../")
        || token.starts_with('/')
    {
        return true;
    }
    // Treat `Schema.org`, `crates.io`, `docs.rs`, `nsm-approach.net`,
    // `learnthesewordsfirst.com` and similar host-like tokens as URLs
    // so the tokenizer doesn't split them into the LHS + a dangling
    // TLD ("org", "io", "rs", …).
    if token.contains('/') && token.contains('.') {
        return true;
    }
    if token.contains('.') {
        let last = token.rsplit('.').next().unwrap_or("");
        let known_tlds = [
            "com", "org", "net", "io", "rs", "co", "uk", "edu", "gov", "info", "dev", "app", "ai",
            "page", "pages", "github",
        ];
        if known_tlds.contains(&last.to_lowercase().as_str()) {
            return true;
        }
    }
    false
}

/// Split on anything that is not a letter or apostrophe. Hyphens
/// **separate** tokens — `cross-ontology` becomes `cross` and
/// `ontology` so that compound English coinages don't all need to be
/// allow-listed separately.
fn tokenize(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in s.chars() {
        if c.is_alphabetic() || c == '\'' {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn normalize(word: &str) -> String {
    let lower = word.to_ascii_lowercase();
    // Trim leading/trailing apostrophes and hyphens.
    let trimmed = lower
        .trim_start_matches(['\'', '-'])
        .trim_end_matches(['\'', '-'])
        .to_string();
    // Strip possessive "'s".
    if let Some(stripped) = trimmed.strip_suffix("'s") {
        return stripped.to_string();
    }
    trimmed
}

fn try_lemmas(word: &str) -> impl Iterator<Item = String> + '_ {
    let candidates = [
        word.strip_suffix("ies").map(|w| format!("{w}y")),
        word.strip_suffix("es").map(str::to_string),
        word.strip_suffix('s').map(str::to_string),
        word.strip_suffix("ed").map(str::to_string),
        word.strip_suffix("ing").map(str::to_string),
        word.strip_suffix("ly").map(str::to_string),
    ];
    candidates.into_iter().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic_prose() {
        let toks = tokenize("Hello, world! It's fine.");
        assert_eq!(toks, vec!["Hello", "world", "It's", "fine"]);
    }

    #[test]
    fn strip_code_spans_and_links() {
        let line = "Use `parse_lino` to read [the lino notation](https://example.com).";
        let cleaned = strip_inline_noise(line);
        assert!(!cleaned.contains("parse_lino"));
        assert!(!cleaned.contains("https"));
        assert!(cleaned.contains("the lino notation"));
    }

    #[test]
    fn normalize_strips_possessive() {
        assert_eq!(normalize("App's"), "app");
    }
}
