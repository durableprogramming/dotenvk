// Core library for dotenvk - a CLI tool for managing .env files.
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

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum EnvLine {
    KeyValue { key: String, value: String },
    Comment(String),
    Empty(String),
}

#[derive(Parser)]
#[command(name = "dotenvk")]
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
                let value = line[eq_pos + 1..].to_string();
                EnvLine::KeyValue { key, value }
            } else {
                EnvLine::Comment(line)
            }
        })
        .collect()
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
        format!("{}\n", content)
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
        let eq_pos = pair.find('=').ok_or_else(|| {
            anyhow::anyhow!("Invalid key=value pair: {}", pair)
        })?;
        let key = pair[..eq_pos].trim().to_string();
        let value = pair[eq_pos + 1..].to_string();
        
        let mut found = false;
        for line in lines.iter_mut() {
            if let EnvLine::KeyValue { key: existing_key, value: existing_value, .. } = line {
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

pub fn generate_random_password(length: usize, include_numeric: bool, include_symbols: bool) -> String {
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
        anyhow::bail!("xkcdpass command failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn shell_escape(value: &str) -> String {
    if value.contains(' ') || value.contains('"') || value.contains('\'') || value.contains('$') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
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
        assert_eq!(result[0], EnvLine::KeyValue { key: "KEY".to_string(), value: "value".to_string() });
        assert_eq!(result[1], EnvLine::Comment("# Comment".to_string()));
        assert_eq!(result[2], EnvLine::Empty("".to_string()));
        assert_eq!(result[3], EnvLine::KeyValue { key: "ANOTHER_KEY".to_string(), value: "another_value".to_string() });
    }

    #[test]
    fn test_parse_env_file_edge_cases() {
        let content = "KEY_WITH_SPACES = value with spaces\nKEY_WITH_EQUALS=value=with=equals\n  # Indented comment";
        let result = parse_env_file(content);
        
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], EnvLine::KeyValue { 
            key: "KEY_WITH_SPACES".to_string(), 
            value: " value with spaces".to_string() 
        });
        assert_eq!(result[1], EnvLine::KeyValue { 
            key: "KEY_WITH_EQUALS".to_string(), 
            value: "value=with=equals".to_string() 
        });
        assert_eq!(result[2], EnvLine::Comment("  # Indented comment".to_string()));
    }

    #[test]
    fn test_write_env_file() {
        let lines = vec![
            EnvLine::KeyValue { key: "KEY1".to_string(), value: "value1".to_string() },
            EnvLine::Comment("# This is a comment".to_string()),
            EnvLine::Empty("".to_string()),
            EnvLine::KeyValue { key: "KEY2".to_string(), value: "value2".to_string() },
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
        let lines = vec![
            EnvLine::KeyValue { key: "KEY".to_string(), value: "value".to_string() },
        ];
        
        let result = write_env_file(&lines);
        assert_eq!(result, "KEY=value\n");
    }

    #[test]
    fn test_set_env_vars_new_key() {
        let mut lines = vec![
            EnvLine::KeyValue { key: "EXISTING".to_string(), value: "value".to_string() }
        ];
        
        set_env_vars(&mut lines, vec!["NEW_KEY=new_value".to_string()]).unwrap();
        
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], EnvLine::KeyValue { 
            key: "NEW_KEY".to_string(), 
            value: "new_value".to_string() 
        });
    }

    #[test]
    fn test_set_env_vars_update_existing() {
        let mut lines = vec![
            EnvLine::KeyValue { key: "KEY".to_string(), value: "old_value".to_string() }
        ];
        
        set_env_vars(&mut lines, vec!["KEY=new_value".to_string()]).unwrap();
        
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], EnvLine::KeyValue { 
            key: "KEY".to_string(), 
            value: "new_value".to_string() 
        });
    }

    #[test]
    fn test_set_env_vars_invalid_format() {
        let mut lines = Vec::new();
        let result = set_env_vars(&mut lines, vec!["invalid_pair".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid key=value pair"));
    }

    #[test]
    fn test_unset_env_vars() {
        let mut lines = vec![
            EnvLine::KeyValue { key: "KEY1".to_string(), value: "value1".to_string() },
            EnvLine::Comment("# Comment".to_string()),
            EnvLine::KeyValue { key: "KEY2".to_string(), value: "value2".to_string() },
            EnvLine::KeyValue { key: "KEY3".to_string(), value: "value3".to_string() },
        ];
        
        unset_env_vars(&mut lines, vec!["KEY1".to_string(), "KEY3".to_string()]);
        
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], EnvLine::Comment("# Comment".to_string()));
        assert_eq!(lines[1], EnvLine::KeyValue { key: "KEY2".to_string(), value: "value2".to_string() });
    }

    #[test]
    fn test_get_env_vars() {
        let lines = vec![
            EnvLine::KeyValue { key: "KEY1".to_string(), value: "value1".to_string() },
            EnvLine::Comment("# Comment".to_string()),
            EnvLine::KeyValue { key: "KEY2".to_string(), value: "value2".to_string() },
        ];
        
        let result = get_env_vars(&lines);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(result.get("KEY2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_get_env_keys() {
        let lines = vec![
            EnvLine::KeyValue { key: "KEY1".to_string(), value: "value1".to_string() },
            EnvLine::Comment("# Comment".to_string()),
            EnvLine::KeyValue { key: "KEY2".to_string(), value: "value2".to_string() },
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

    #[test]
    fn test_shell_escape() {
        assert_eq!(shell_escape("simple"), "simple");
        assert_eq!(shell_escape("with space"), "\"with space\"");
        assert_eq!(shell_escape("with\"quote"), "\"with\\\"quote\"");
        assert_eq!(shell_escape("with'apostrophe"), "\"with'apostrophe\"");
        assert_eq!(shell_escape("with$dollar"), "\"with$dollar\"");
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.