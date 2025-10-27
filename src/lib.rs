// Core library for durable-appconfig-dotenv - a CLI tool for managing .env files.
//
// This module provides functionality for parsing, modifying, and writing environment files
// while preserving their structure (comments, empty lines, and formatting). It includes
// support for setting/unsetting variables, generating secure passwords, and exporting
// configurations in multiple formats.
//
// Key features:
// - Structure-preserving .env file parsing and writing
// - Secure random password generation with customizable character sets
// - XKCD-style passphrase generation (requires external xkcdpass command)
// - Export to bash and JSON formats with proper shell escaping
// - Safe file operations with comprehensive error handling
//

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub enum EnvLine {
    KeyValue { key: String, value: String },
    Comment(String),
    Empty(String),
}

#[derive(Parser)]
#[command(name = "durable-appconfig-dotenv")]
#[command(about = "A CLI tool for editing .env files")]
pub struct Cli {
    #[arg(short, long, default_value = ".env")]
    pub file: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set one or more key=value pairs in the .env file
    Set {
        /// Key=value pairs to set
        pairs: Vec<String>,
    },
    /// Remove one or more keys from the .env file
    Unset {
        /// Keys to remove
        keys: Vec<String>,
    },
    /// Export the .env file as bash export statements or JSON
    Export {
        /// Output format: bash or json
        #[arg(short, long, default_value = "bash")]
        format: String,
    },
    /// List all keys from the .env file
    Keys,
    /// Generate secure random passwords and set them for specified keys
    Randomize {
        /// Keys to set with random passwords
        keys: Vec<String>,
        /// Include numeric characters (0-9)
        #[arg(long)]
        numeric: bool,
        /// Include symbol characters (!@#$%^&*()_+-=[]{}|;:,.<>?)
        #[arg(long)]
        symbol: bool,
        /// Password length (default: 32)
        #[arg(short, long, default_value = "32")]
        length: usize,
        /// Generate XKCD-style passphrase using xkcdpass command
        #[arg(long)]
        xkcd: bool,
    },
}

pub fn parse_env_file(content: &str) -> Vec<EnvLine> {
    content
        .lines()
        .map(|line| {
            let line = line.to_string();
            if line.trim().is_empty() {
                EnvLine::Empty(line)
            } else if line.trim_start().starts_with('#') {
                EnvLine::Comment(line)
            } else if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let raw_value = &line[eq_pos + 1..];
                let value = parse_value(raw_value);
                EnvLine::KeyValue { key, value }
            } else {
                EnvLine::Comment(line)
            }
        })
        .collect()
}

/// Parse a value according to dotenv format rules:
/// - Double-quoted values: strips quotes and processes escape sequences
/// - Single-quoted values: strips quotes, no escape processing (literal)
/// - Unquoted values: trims whitespace, stops at # comment
fn parse_value(raw: &str) -> String {
    let trimmed = raw.trim_start();

    if trimmed.is_empty() {
        return String::new();
    }

    // Double-quoted value
    if trimmed.starts_with('"') {
        return parse_double_quoted(trimmed);
    }

    // Single-quoted value
    if trimmed.starts_with('\'') {
        return parse_single_quoted(trimmed);
    }

    // Unquoted value - find end (stops at unquoted # or end of line)
    parse_unquoted(trimmed)
}

fn parse_double_quoted(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().skip(1); // skip opening quote
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if escaped {
            match ch {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                '\'' => result.push('\''),
                _ => {
                    // Unknown escape sequence - keep backslash and character
                    result.push('\\');
                    result.push(ch);
                }
            }
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            // End of quoted string
            break;
        } else {
            result.push(ch);
        }
    }

    result
}

fn parse_single_quoted(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().skip(1); // skip opening quote

    while let Some(ch) = chars.next() {
        if ch == '\'' {
            // End of quoted string
            break;
        }
        result.push(ch);
    }

    result
}

fn parse_unquoted(s: &str) -> String {
    // Find the first unquoted # to handle inline comments
    let mut result = String::new();

    for ch in s.chars() {
        if ch == '#' {
            break;
        }
        result.push(ch);
    }

    result.trim_end().to_string()
}

pub fn write_env_file(lines: &[EnvLine]) -> String {
    let content = lines
        .iter()
        .map(|line| match line {
            EnvLine::KeyValue { key, value, .. } => format!("{key}={value}"),
            EnvLine::Comment(content) => content.clone(),
            EnvLine::Empty(content) => content.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Ensure file ends with a newline
    if content.is_empty() {
        content
    } else {
        format!("{content}\n")
    }
}

pub fn read_env_file(file_path: &PathBuf) -> Result<Vec<EnvLine>> {
    if file_path.exists() {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        Ok(parse_env_file(&content))
    } else {
        Ok(Vec::new())
    }
}

pub fn save_env_file(file_path: &PathBuf, lines: &[EnvLine]) -> Result<()> {
    let content = write_env_file(lines);
    fs::write(file_path, content)
        .with_context(|| format!("Failed to write file: {}", file_path.display()))
}

pub fn set_env_vars(lines: &mut Vec<EnvLine>, pairs: Vec<String>) -> Result<()> {
    for pair in pairs {
        let eq_pos = pair
            .find('=')
            .ok_or_else(|| anyhow::anyhow!("Invalid key=value pair: {}", pair))?;
        let key = pair[..eq_pos].trim().to_string();
        let value = pair[eq_pos + 1..].to_string();

        let mut found = false;
        for line in lines.iter_mut() {
            if let EnvLine::KeyValue {
                key: existing_key,
                value: existing_value,
                ..
            } = line
            {
                if existing_key == &key {
                    *existing_value = value.clone();
                    found = true;
                    break;
                }
            }
        }

        if !found {
            lines.push(EnvLine::KeyValue { key, value });
        }
    }
    Ok(())
}

pub fn unset_env_vars(lines: &mut Vec<EnvLine>, keys: Vec<String>) {
    lines.retain(|line| {
        if let EnvLine::KeyValue { key, .. } = line {
            !keys.contains(key)
        } else {
            true
        }
    });
}

pub fn get_env_vars(lines: &[EnvLine]) -> HashMap<String, String> {
    lines
        .iter()
        .filter_map(|line| {
            if let EnvLine::KeyValue { key, value, .. } = line {
                Some((key.clone(), value.clone()))
            } else {
                None
            }
        })
        .collect()
}

pub fn get_env_keys(lines: &[EnvLine]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            if let EnvLine::KeyValue { key, .. } = line {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect()
}

pub fn generate_random_password(
    length: usize,
    include_numeric: bool,
    include_symbols: bool,
) -> String {
    let mut charset: Vec<u8> = Vec::new();

    // Always include letters
    charset.extend(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ");

    if include_numeric {
        charset.extend(b"0123456789");
    }

    if include_symbols {
        charset.extend(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
    }

    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

pub fn generate_xkcd_password() -> Result<String> {
    let output = Command::new("xkcdpass")
        .arg("-d-")
        .output()
        .context("Failed to execute xkcdpass command. Make sure it's installed.")?;

    if !output.status.success() {
        anyhow::bail!(
            "xkcdpass command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Escape a value for safe use in bash export statements
/// Handles special characters, quotes, backslashes, and newlines
pub fn shell_escape(value: &str) -> String {
    // Check if escaping is needed
    let needs_escaping = value.chars().any(|c| {
        matches!(
            c,
            ' ' | '\t'
                | '\n'
                | '\r'
                | '"'
                | '\''
                | '$'
                | '`'
                | '\\'
                | '!'
                | '*'
                | '?'
                | '['
                | ']'
                | '{'
                | '}'
                | '('
                | ')'
                | '<'
                | '>'
                | '&'
                | '|'
                | ';'
                | '#'
        )
    });

    if !needs_escaping {
        return value.to_string();
    }

    // Use double quotes and escape special characters within
    let mut result = String::from('"');

    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '$' => result.push_str("\\$"),
            '`' => result.push_str("\\`"),
            '!' => result.push_str("\\!"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(ch),
        }
    }

    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_env_file_basic() {
        let content = "KEY=value\n# Comment\n\nANOTHER_KEY=another_value";
        let result = parse_env_file(content);

        assert_eq!(result.len(), 4);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "KEY".to_string(),
                value: "value".to_string()
            }
        );
        assert_eq!(result[1], EnvLine::Comment("# Comment".to_string()));
        assert_eq!(result[2], EnvLine::Empty("".to_string()));
        assert_eq!(
            result[3],
            EnvLine::KeyValue {
                key: "ANOTHER_KEY".to_string(),
                value: "another_value".to_string()
            }
        );
    }

    #[test]
    fn test_parse_env_file_edge_cases() {
        let content = "KEY_WITH_SPACES = value with spaces\nKEY_WITH_EQUALS=value=with=equals\n  # Indented comment";
        let result = parse_env_file(content);

        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "KEY_WITH_SPACES".to_string(),
                value: "value with spaces".to_string()
            }
        );
        assert_eq!(
            result[1],
            EnvLine::KeyValue {
                key: "KEY_WITH_EQUALS".to_string(),
                value: "value=with=equals".to_string()
            }
        );
        assert_eq!(
            result[2],
            EnvLine::Comment("  # Indented comment".to_string())
        );
    }

    #[test]
    fn test_write_env_file() {
        let lines = vec![
            EnvLine::KeyValue {
                key: "KEY1".to_string(),
                value: "value1".to_string(),
            },
            EnvLine::Comment("# This is a comment".to_string()),
            EnvLine::Empty("".to_string()),
            EnvLine::KeyValue {
                key: "KEY2".to_string(),
                value: "value2".to_string(),
            },
        ];

        let result = write_env_file(&lines);
        assert_eq!(result, "KEY1=value1\n# This is a comment\n\nKEY2=value2\n");
    }

    #[test]
    fn test_write_env_file_empty() {
        let lines = vec![];
        let result = write_env_file(&lines);
        assert_eq!(result, "");
    }

    #[test]
    fn test_write_env_file_single_line() {
        let lines = vec![EnvLine::KeyValue {
            key: "KEY".to_string(),
            value: "value".to_string(),
        }];

        let result = write_env_file(&lines);
        assert_eq!(result, "KEY=value\n");
    }

    #[test]
    fn test_set_env_vars_new_key() {
        let mut lines = vec![EnvLine::KeyValue {
            key: "EXISTING".to_string(),
            value: "value".to_string(),
        }];

        set_env_vars(&mut lines, vec!["NEW_KEY=new_value".to_string()]).unwrap();

        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines[1],
            EnvLine::KeyValue {
                key: "NEW_KEY".to_string(),
                value: "new_value".to_string()
            }
        );
    }

    #[test]
    fn test_set_env_vars_update_existing() {
        let mut lines = vec![EnvLine::KeyValue {
            key: "KEY".to_string(),
            value: "old_value".to_string(),
        }];

        set_env_vars(&mut lines, vec!["KEY=new_value".to_string()]).unwrap();

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0],
            EnvLine::KeyValue {
                key: "KEY".to_string(),
                value: "new_value".to_string()
            }
        );
    }

    #[test]
    fn test_set_env_vars_invalid_format() {
        let mut lines = Vec::new();
        let result = set_env_vars(&mut lines, vec!["invalid_pair".to_string()]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid key=value pair"));
    }

    #[test]
    fn test_unset_env_vars() {
        let mut lines = vec![
            EnvLine::KeyValue {
                key: "KEY1".to_string(),
                value: "value1".to_string(),
            },
            EnvLine::Comment("# Comment".to_string()),
            EnvLine::KeyValue {
                key: "KEY2".to_string(),
                value: "value2".to_string(),
            },
            EnvLine::KeyValue {
                key: "KEY3".to_string(),
                value: "value3".to_string(),
            },
        ];

        unset_env_vars(&mut lines, vec!["KEY1".to_string(), "KEY3".to_string()]);

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], EnvLine::Comment("# Comment".to_string()));
        assert_eq!(
            lines[1],
            EnvLine::KeyValue {
                key: "KEY2".to_string(),
                value: "value2".to_string()
            }
        );
    }

    #[test]
    fn test_get_env_vars() {
        let lines = vec![
            EnvLine::KeyValue {
                key: "KEY1".to_string(),
                value: "value1".to_string(),
            },
            EnvLine::Comment("# Comment".to_string()),
            EnvLine::KeyValue {
                key: "KEY2".to_string(),
                value: "value2".to_string(),
            },
        ];

        let result = get_env_vars(&lines);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_get_env_keys() {
        let lines = vec![
            EnvLine::KeyValue {
                key: "KEY1".to_string(),
                value: "value1".to_string(),
            },
            EnvLine::Comment("# Comment".to_string()),
            EnvLine::KeyValue {
                key: "KEY2".to_string(),
                value: "value2".to_string(),
            },
        ];

        let result = get_env_keys(&lines);
        assert_eq!(result, vec!["KEY1", "KEY2"]);
    }

    #[test]
    fn test_generate_random_password_length() {
        let password = generate_random_password(10, false, false);
        assert_eq!(password.len(), 10);
        assert!(password.chars().all(|c| c.is_ascii_alphabetic()));
    }

    #[test]
    fn test_generate_random_password_with_numeric() {
        let password = generate_random_password(8, true, false);
        assert_eq!(password.len(), 8);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_generate_random_password_with_symbols() {
        let password = generate_random_password(6, false, true);
        assert_eq!(password.len(), 6);
    }

    // ==================== DOTENV FORMAT COMPLIANCE TESTS ====================
    // Tests based on https://github.com/motdotla/dotenv specification

    #[test]
    fn test_parse_basic_key_value() {
        let content = "KEY=value";
        let result = parse_env_file(content);
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "KEY".to_string(),
                value: "value".to_string()
            }
        );
    }

    #[test]
    fn test_parse_empty_value() {
        let content = "EMPTY=";
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "EMPTY".to_string(),
                value: "".to_string()
            }
        );
    }

    #[test]
    fn test_parse_double_quoted_value() {
        let content = r#"DOUBLE_QUOTED="hello world""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "DOUBLE_QUOTED".to_string(),
                value: "hello world".to_string()
            }
        );
    }

    #[test]
    fn test_parse_single_quoted_value() {
        let content = "SINGLE_QUOTED='hello world'";
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "SINGLE_QUOTED".to_string(),
                value: "hello world".to_string()
            }
        );
    }

    #[test]
    fn test_parse_double_quoted_with_escaped_quotes() {
        let content = r#"ESCAPED="She said \"hello\"""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "ESCAPED".to_string(),
                value: r#"She said "hello""#.to_string()
            }
        );
    }

    #[test]
    fn test_parse_double_quoted_with_newline() {
        let content = r#"MULTILINE="line1\nline2\nline3""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "MULTILINE".to_string(),
                value: "line1\nline2\nline3".to_string()
            }
        );
    }

    #[test]
    fn test_parse_double_quoted_with_escaped_backslash() {
        let content = r#"BACKSLASH="C:\\path\\to\\file""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "BACKSLASH".to_string(),
                value: r"C:\path\to\file".to_string()
            }
        );
    }

    #[test]
    fn test_parse_single_quoted_no_escape_processing() {
        let content = r#"LITERAL='Line1\nLine2'"#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "LITERAL".to_string(),
                value: r"Line1\nLine2".to_string()
            }
        );
    }

    #[test]
    fn test_parse_unquoted_with_inline_comment() {
        let content = "VALUE=hello # this is a comment";
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "VALUE".to_string(),
                value: "hello".to_string()
            }
        );
    }

    #[test]
    fn test_parse_quoted_with_hash_inside() {
        let content = r#"SECRET="value#with#hashes""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "SECRET".to_string(),
                value: "value#with#hashes".to_string()
            }
        );
    }

    #[test]
    fn test_parse_special_characters_in_double_quotes() {
        let content = r#"SPECIAL="!@#$%^&*()_+-=[]{}|;:,.<>?""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "SPECIAL".to_string(),
                value: "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string()
            }
        );
    }

    #[test]
    fn test_parse_dollar_sign_in_quotes() {
        let content = r#"DOLLAR="$HOME is a variable""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "DOLLAR".to_string(),
                value: "$HOME is a variable".to_string()
            }
        );
    }

    #[test]
    fn test_parse_backticks_in_quotes() {
        let content = r#"BACKTICK="`echo test`""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "BACKTICK".to_string(),
                value: "`echo test`".to_string()
            }
        );
    }

    #[test]
    fn test_parse_whitespace_around_equals() {
        let content = "KEY = value";
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "KEY".to_string(),
                value: "value".to_string()
            }
        );
    }

    #[test]
    fn test_parse_equals_in_value() {
        let content = "CONNECTION=host=localhost;port=5432";
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "CONNECTION".to_string(),
                value: "host=localhost;port=5432".to_string()
            }
        );
    }

    #[test]
    fn test_parse_comment_line() {
        let content = "# This is a comment";
        let result = parse_env_file(content);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], EnvLine::Comment("# This is a comment".to_string()));
    }

    #[test]
    fn test_parse_empty_line() {
        let content = "\n";
        let result = parse_env_file(content);
        // .lines() on "\n" produces 2 empty strings
        assert!(result.len() >= 1);
        assert_eq!(result[0], EnvLine::Empty("".to_string()));
    }

    #[test]
    fn test_parse_mixed_content() {
        let content = r#"# Config file
DATABASE_URL="postgres://user:pass@localhost/db"

# API Keys
API_KEY=secret123
DEBUG=true # enable debug mode
"#;
        let result = parse_env_file(content);
        assert_eq!(result.len(), 6);
        assert_eq!(result[0], EnvLine::Comment("# Config file".to_string()));
        assert_eq!(
            result[1],
            EnvLine::KeyValue {
                key: "DATABASE_URL".to_string(),
                value: "postgres://user:pass@localhost/db".to_string()
            }
        );
        assert_eq!(result[2], EnvLine::Empty("".to_string()));
        assert_eq!(result[3], EnvLine::Comment("# API Keys".to_string()));
    }

    #[test]
    fn test_parse_escape_sequences() {
        let content = r#"TAB="hello\tworld"
NEWLINE="hello\nworld"
RETURN="hello\rworld"
QUOTE="say \"hi\""
"#;
        let result = parse_env_file(content);

        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "TAB".to_string(),
                value: "hello\tworld".to_string()
            }
        );
        assert_eq!(
            result[1],
            EnvLine::KeyValue {
                key: "NEWLINE".to_string(),
                value: "hello\nworld".to_string()
            }
        );
        assert_eq!(
            result[2],
            EnvLine::KeyValue {
                key: "RETURN".to_string(),
                value: "hello\rworld".to_string()
            }
        );
        assert_eq!(
            result[3],
            EnvLine::KeyValue {
                key: "QUOTE".to_string(),
                value: r#"say "hi""#.to_string()
            }
        );
    }

    #[test]
    fn test_parse_unicode_characters() {
        let content = r#"EMOJI="Hello 👋 World 🌍"
UNICODE="Ñoño en español"
"#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "EMOJI".to_string(),
                value: "Hello 👋 World 🌍".to_string()
            }
        );
        assert_eq!(
            result[1],
            EnvLine::KeyValue {
                key: "UNICODE".to_string(),
                value: "Ñoño en español".to_string()
            }
        );
    }

    #[test]
    fn test_parse_url_values() {
        let content = r#"API_URL="https://api.example.com/v1?key=value&foo=bar"
WEBHOOK="http://example.com/webhook?token=abc123"
"#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "API_URL".to_string(),
                value: "https://api.example.com/v1?key=value&foo=bar".to_string()
            }
        );
    }

    #[test]
    fn test_parse_json_values() {
        let content = r#"JSON_CONFIG="{\"name\":\"test\",\"value\":123}""#;
        let result = parse_env_file(content);
        assert_eq!(
            result[0],
            EnvLine::KeyValue {
                key: "JSON_CONFIG".to_string(),
                value: r#"{"name":"test","value":123}"#.to_string()
            }
        );
    }

    // ==================== SHELL ESCAPE TESTS ====================

    #[test]
    fn test_shell_escape_simple() {
        assert_eq!(shell_escape("simple"), "simple");
        assert_eq!(shell_escape("simple123"), "simple123");
    }

    #[test]
    fn test_shell_escape_with_space() {
        assert_eq!(shell_escape("with space"), "\"with space\"");
    }

    #[test]
    fn test_shell_escape_with_double_quote() {
        assert_eq!(shell_escape(r#"with"quote"#), r#""with\"quote""#);
    }

    #[test]
    fn test_shell_escape_with_single_quote() {
        assert_eq!(shell_escape("with'apostrophe"), r#""with'apostrophe""#);
    }

    #[test]
    fn test_shell_escape_with_dollar() {
        assert_eq!(shell_escape("with$dollar"), r#""with\$dollar""#);
    }

    #[test]
    fn test_shell_escape_with_backtick() {
        assert_eq!(shell_escape("with`backtick"), r#""with\`backtick""#);
    }

    #[test]
    fn test_shell_escape_with_backslash() {
        assert_eq!(shell_escape(r"with\backslash"), r#""with\\backslash""#);
    }

    #[test]
    fn test_shell_escape_with_newline() {
        assert_eq!(shell_escape("with\newline"), r#""with\newline""#);
    }

    #[test]
    fn test_shell_escape_with_tab() {
        assert_eq!(shell_escape("with\ttab"), r#""with\ttab""#);
    }

    #[test]
    fn test_shell_escape_with_exclamation() {
        assert_eq!(shell_escape("with!exclamation"), r#""with\!exclamation""#);
    }

    #[test]
    fn test_shell_escape_complex() {
        let complex = r#"She said "Hello $USER", then ran `echo test`"#;
        let escaped = shell_escape(complex);
        assert_eq!(
            escaped,
            r#""She said \"Hello \$USER\", then ran \`echo test\`""#
        );
    }

    #[test]
    fn test_shell_escape_url() {
        let url = "https://api.example.com/v1?key=value&foo=bar";
        let escaped = shell_escape(url);
        assert_eq!(escaped, r#""https://api.example.com/v1?key=value&foo=bar""#);
    }

    #[test]
    fn test_shell_escape_json() {
        let json = r#"{"name":"test","value":123}"#;
        let escaped = shell_escape(json);
        assert_eq!(escaped, r#""{\"name\":\"test\",\"value\":123}""#);
    }

    #[test]
    fn test_shell_escape_semicolon() {
        assert_eq!(shell_escape("cmd1;cmd2"), r#""cmd1;cmd2""#);
    }

    #[test]
    fn test_shell_escape_pipe() {
        assert_eq!(shell_escape("cmd1|cmd2"), r#""cmd1|cmd2""#);
    }

    #[test]
    fn test_shell_escape_ampersand() {
        assert_eq!(shell_escape("cmd&"), r#""cmd&""#);
    }

    #[test]
    fn test_shell_escape_redirects() {
        assert_eq!(shell_escape("file>output"), r#""file>output""#);
        assert_eq!(shell_escape("file<input"), r#""file<input""#);
    }

    #[test]
    fn test_shell_escape_wildcards() {
        assert_eq!(shell_escape("*.txt"), r#""*.txt""#);
        assert_eq!(shell_escape("file?.dat"), r#""file?.dat""#);
    }

    #[test]
    fn test_shell_escape_brackets() {
        assert_eq!(shell_escape("[a-z]"), r#""[a-z]""#);
        assert_eq!(shell_escape("{a,b}"), r#""{a,b}""#);
    }

    #[test]
    fn test_shell_escape_parentheses() {
        assert_eq!(shell_escape("(test)"), r#""(test)""#);
    }

    #[test]
    fn test_shell_escape_hash() {
        assert_eq!(shell_escape("value#comment"), r#""value#comment""#);
    }

    // ==================== COMPREHENSIVE EXPORT BASH TESTS ====================
    // Tests to ensure export function produces valid bash code

    #[test]
    fn test_export_bash_simple_values() {
        let lines = vec![
            EnvLine::KeyValue {
                key: "SIMPLE".to_string(),
                value: "simple".to_string(),
            },
            EnvLine::KeyValue {
                key: "NUMERIC".to_string(),
                value: "12345".to_string(),
            },
            EnvLine::KeyValue {
                key: "UNDERSCORE".to_string(),
                value: "under_score".to_string(),
            },
        ];

        let env_vars = get_env_vars(&lines);
        for (key, value) in env_vars {
            let export_line = format!("export {key}={}", shell_escape(&value));
            // Simple values should not need quotes
            assert!(
                !export_line.contains('"'),
                "Simple value should not be quoted: {export_line}"
            );
        }
    }

    #[test]
    fn test_export_bash_with_spaces() {
        let value = "value with spaces";
        let escaped = shell_escape(value);
        let export_line = format!("export KEY={escaped}");
        assert_eq!(export_line, r#"export KEY="value with spaces""#);
    }

    #[test]
    fn test_export_bash_with_quotes() {
        let value = r#"She said "hello""#;
        let escaped = shell_escape(value);
        let export_line = format!("export KEY={escaped}");
        assert_eq!(export_line, r#"export KEY="She said \"hello\"""#);
    }

    #[test]
    fn test_export_bash_with_dollar_sign() {
        let value = "$HOME/path";
        let escaped = shell_escape(value);
        let export_line = format!("export PATH_VAR={escaped}");
        assert_eq!(export_line, r#"export PATH_VAR="\$HOME/path""#);

        // Ensure $ is properly escaped
        assert!(escaped.contains(r"\$"), "Dollar sign must be escaped");
    }

    #[test]
    fn test_export_bash_with_backtick() {
        let value = "`echo test`";
        let escaped = shell_escape(value);
        let export_line = format!("export CMD={escaped}");
        assert_eq!(export_line, r#"export CMD="\`echo test\`""#);

        // Ensure backtick is properly escaped
        assert!(escaped.contains(r"\`"), "Backtick must be escaped");
    }

    #[test]
    fn test_export_bash_with_backslash() {
        let value = r"C:\path\to\file";
        let escaped = shell_escape(value);
        let export_line = format!("export FILEPATH={escaped}");
        assert_eq!(export_line, r#"export FILEPATH="C:\\path\\to\\file""#);
    }

    #[test]
    fn test_export_bash_with_newline() {
        let value = "line1\nline2\nline3";
        let escaped = shell_escape(value);
        let export_line = format!("export MULTILINE={escaped}");
        assert_eq!(export_line, "export MULTILINE=\"line1\\nline2\\nline3\"");
    }

    #[test]
    fn test_export_bash_with_tabs() {
        let value = "col1\tcol2\tcol3";
        let escaped = shell_escape(value);
        let export_line = format!("export TABBED={escaped}");
        assert_eq!(export_line, "export TABBED=\"col1\\tcol2\\tcol3\"");
    }

    #[test]
    fn test_export_bash_with_carriage_return() {
        let value = "line1\rline2";
        let escaped = shell_escape(value);
        let export_line = format!("export CR={escaped}");
        assert_eq!(export_line, "export CR=\"line1\\rline2\"");
    }

    #[test]
    fn test_export_bash_with_exclamation() {
        let value = "alert!";
        let escaped = shell_escape(value);
        let export_line = format!("export ALERT={escaped}");
        assert_eq!(export_line, r#"export ALERT="alert\!""#);
    }

    #[test]
    fn test_export_bash_command_injection_prevention() {
        // Test various command injection attempts
        let dangerous_values = vec![
            ("$(whoami)", r#""\$(whoami)""#),
            ("`whoami`", r#""\`whoami\`""#),
            ("; rm -rf /", r#""; rm -rf /""#),
            ("| cat /etc/passwd", r#""| cat /etc/passwd""#),
            ("&& ls -la", r#""&& ls -la""#),
        ];

        for (dangerous, expected_escaped) in dangerous_values {
            let escaped = shell_escape(dangerous);
            assert_eq!(
                escaped, expected_escaped,
                "Failed to properly escape: {dangerous}"
            );
            // Ensure the dangerous characters are contained within quotes
            assert!(
                escaped.starts_with('"') && escaped.ends_with('"'),
                "Dangerous value must be quoted: {dangerous}"
            );
        }
    }

    #[test]
    fn test_export_bash_url_values() {
        let url = "https://api.example.com/v1?key=value&token=abc123";
        let escaped = shell_escape(url);
        let export_line = format!("export API_URL={escaped}");
        assert_eq!(
            export_line,
            r#"export API_URL="https://api.example.com/v1?key=value&token=abc123""#
        );
    }

    #[test]
    fn test_export_bash_json_values() {
        let json = r#"{"name":"test","value":123,"nested":{"key":"val"}}"#;
        let escaped = shell_escape(json);
        let export_line = format!("export JSON_CONFIG={escaped}");
        assert_eq!(
            export_line,
            r#"export JSON_CONFIG="{\"name\":\"test\",\"value\":123,\"nested\":{\"key\":\"val\"}}""#
        );
    }

    #[test]
    fn test_export_bash_database_connection_string() {
        let conn = "postgresql://user:p@ss$word@localhost:5432/dbname?sslmode=require";
        let escaped = shell_escape(conn);
        let _export_line = format!("export DATABASE_URL={escaped}");
        // Should escape $ in password
        assert!(escaped.contains(r"\$"));
        assert!(escaped.contains("?"));
        assert!(escaped.contains("&") || escaped.contains(":"));
    }

    #[test]
    fn test_export_bash_redis_url() {
        let redis = "redis://:p@ssw0rd!@localhost:6379/0";
        let escaped = shell_escape(redis);
        let _export_line = format!("export REDIS_URL={escaped}");
        // Should properly escape the ! in password
        assert!(escaped.contains(r"\!"));
    }

    #[test]
    fn test_export_bash_empty_string() {
        let value = "";
        let escaped = shell_escape(value);
        let export_line = format!("export EMPTY={escaped}");
        assert_eq!(export_line, "export EMPTY=");
    }

    #[test]
    fn test_export_bash_whitespace_only() {
        let value = "   ";
        let escaped = shell_escape(value);
        let export_line = format!("export SPACES={escaped}");
        assert_eq!(export_line, r#"export SPACES="   ""#);
    }

    #[test]
    fn test_export_bash_special_chars_combination() {
        let value = r#"$USER: "Hello!", path=`pwd`, cmd;ls&echo"#;
        let escaped = shell_escape(value);
        let _export_line = format!("export COMPLEX={escaped}");

        // Verify all special chars are escaped
        assert!(escaped.contains(r"\$"));
        assert!(escaped.contains(r#"\""#));
        assert!(escaped.contains(r"\!"));
        assert!(escaped.contains(r"\`"));
        assert!(escaped.contains(";"));
        assert!(escaped.contains("&"));
    }

    #[test]
    fn test_export_bash_unicode_emoji() {
        let value = "Hello 👋 World 🌍";
        let escaped = shell_escape(value);
        let export_line = format!("export EMOJI={escaped}");
        assert_eq!(export_line, r#"export EMOJI="Hello 👋 World 🌍""#);
    }

    #[test]
    fn test_export_bash_unicode_special() {
        let value = "Ñoño en español — Ñoño";
        let escaped = shell_escape(value);
        let export_line = format!("export UNICODE={escaped}");
        assert_eq!(export_line, r#"export UNICODE="Ñoño en español — Ñoño""#);
    }

    #[test]
    fn test_export_bash_wildcard_characters() {
        let wildcards = vec![
            ("*.txt", r#""*.txt""#),
            ("file?.dat", r#""file?.dat""#),
            ("[a-z]*", r#""[a-z]*""#),
            ("{a,b,c}", r#""{a,b,c}""#),
        ];

        for (value, expected) in wildcards {
            let escaped = shell_escape(value);
            assert_eq!(
                escaped, expected,
                "Failed to properly escape wildcard: {value}"
            );
        }
    }

    #[test]
    fn test_export_bash_redirect_operators() {
        let value = "output>file<input";
        let escaped = shell_escape(value);
        let export_line = format!("export REDIRECT={escaped}");
        assert_eq!(export_line, r#"export REDIRECT="output>file<input""#);
    }

    #[test]
    fn test_export_bash_all_special_symbols() {
        let value = "!@#$%^&*()_+-=[]{}|;:,.<>?";
        let escaped = shell_escape(value);
        let _export_line = format!("export SYMBOLS={escaped}");

        // Should be quoted and exclamation should be escaped
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
        assert!(escaped.contains(r"\!"));
    }

    #[test]
    fn test_export_bash_ssh_key_format() {
        let value = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC... user@host";
        let escaped = shell_escape(value);
        // Should be quoted due to spaces
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
    }

    #[test]
    fn test_export_bash_jwt_token() {
        let value = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let escaped = shell_escape(value);
        // Should NOT need escaping (alphanumeric + dots + underscores + hyphens)
        assert_eq!(escaped, value);
    }

    #[test]
    fn test_export_bash_base64_encoded() {
        let value = "SGVsbG8gV29ybGQh";
        let escaped = shell_escape(value);
        // Base64 should not need escaping
        assert_eq!(escaped, value);
    }

    #[test]
    fn test_export_bash_path_with_spaces() {
        let value = "/path/to/My Documents/file.txt";
        let escaped = shell_escape(value);
        let export_line = format!("export PATH_VAR={escaped}");
        assert_eq!(export_line, r#"export PATH_VAR="/path/to/My Documents/file.txt""#);
    }

    #[test]
    fn test_export_bash_windows_path() {
        let value = r"C:\Program Files\MyApp\bin";
        let escaped = shell_escape(value);
        let export_line = format!("export WIN_PATH={escaped}");
        assert_eq!(export_line, r#"export WIN_PATH="C:\\Program Files\\MyApp\\bin""#);
    }

    #[test]
    fn test_export_bash_email_address() {
        let value = "user+tag@example.com";
        let escaped = shell_escape(value);
        // Should not need escaping
        assert_eq!(escaped, value);
    }

    #[test]
    fn test_export_bash_ipv6_address() {
        let value = "2001:0db8:85a3:0000:0000:8a2e:0370:7334";
        let escaped = shell_escape(value);
        // Should not need escaping
        assert_eq!(escaped, value);
    }

    #[test]
    fn test_export_bash_multiple_quotes_nested() {
        let value = r#"outer "middle 'inner' middle" outer"#;
        let escaped = shell_escape(value);
        let export_line = format!("export NESTED={escaped}");
        assert_eq!(
            export_line,
            r#"export NESTED="outer \"middle 'inner' middle\" outer""#
        );
    }

    #[test]
    fn test_export_bash_regex_patterns() {
        let value = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let escaped = shell_escape(value);
        // Should be quoted due to special chars
        assert!(escaped.starts_with('"'));
        assert!(escaped.ends_with('"'));
        assert!(escaped.contains(r"\\"));
    }

    #[test]
    fn test_export_bash_xml_fragment() {
        let value = r#"<root attr="value">text</root>"#;
        let escaped = shell_escape(value);
        let _export_line = format!("export XML={escaped}");
        // Should escape the quotes
        assert!(escaped.contains(r#"\""#));
        assert!(escaped.contains("<"));
        assert!(escaped.contains(">"));
    }

    #[test]
    fn test_export_bash_sql_query() {
        let value = r#"SELECT * FROM users WHERE name = "John" AND age > 25;"#;
        let escaped = shell_escape(value);
        // Should escape quotes and handle semicolon
        assert!(escaped.contains(r#"\""#));
        assert!(escaped.contains(";"));
    }

    #[test]
    fn test_export_bash_escaped_sequences_preservation() {
        // Test that our escaping preserves the intended meaning
        let test_cases = vec![
            ("\n", "\\n"),      // newline should become \n in output
            ("\t", "\\t"),      // tab should become \t
            ("\r", "\\r"),      // carriage return should become \r
            ("\\", "\\\\"),     // backslash should be doubled
            ("\"", "\\\""),     // quote should be escaped
            ("$", "\\$"),       // dollar should be escaped
            ("`", "\\`"),       // backtick should be escaped
            ("!", "\\!"),       // exclamation should be escaped
        ];

        for (input, expected_in_output) in test_cases {
            let escaped = shell_escape(input);
            assert!(
                escaped.contains(expected_in_output),
                "Expected {expected_in_output} in escaped output for input {input:?}, got: {escaped}"
            );
        }
    }
}


// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
