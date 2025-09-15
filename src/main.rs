// Main entry point for dotenvk - a CLI tool for managing .env files.
//
// This module implements the command-line interface and dispatches commands to the appropriate
// handlers in the library module. It provides subcommands for setting/unsetting environment
// variables, generating secure passwords, listing keys, and exporting configurations.
//
// The CLI supports operations on custom .env files via the --file flag and maintains file
// structure preservation while performing modifications. All operations include proper error
// handling with contextual messages for better user experience.

use dotenvk::*;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;


fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Set { pairs } => set_command(&cli.file, pairs),
        Commands::Unset { keys } => unset_command(&cli.file, keys),
        Commands::Export { format } => export_command(&cli.file, &format),
        Commands::Keys => keys_command(&cli.file),
        Commands::Randomize { keys, numeric, symbol, length, xkcd } => {
            randomize_command(&cli.file, keys, numeric, symbol, length, xkcd)
        },
    }
}


fn set_command(file_path: &PathBuf, pairs: Vec<String>) -> Result<()> {
    let mut lines = read_env_file(file_path)?;
    set_env_vars(&mut lines, pairs)?;
    save_env_file(file_path, &lines)
}

fn unset_command(file_path: &PathBuf, keys: Vec<String>) -> Result<()> {
    let mut lines = read_env_file(file_path)?;
    unset_env_vars(&mut lines, keys);
    save_env_file(file_path, &lines)
}

fn export_command(file_path: &PathBuf, format: &str) -> Result<()> {
    let lines = read_env_file(file_path)?;
    let env_vars = get_env_vars(&lines);
    
    match format.to_lowercase().as_str() {
        "bash" => {
            for (key, value) in env_vars {
                println!("export {key}={}", shell_escape(&value));
            }
        }
        "json" => {
            let json = serde_json::to_string_pretty(&env_vars)
                .context("Failed to serialize to JSON")?;
            println!("{json}");
        }
        _ => {
            anyhow::bail!("Unsupported format: {}. Use 'bash' or 'json'", format);
        }
    }
    
    Ok(())
}

fn keys_command(file_path: &PathBuf) -> Result<()> {
    let lines = read_env_file(file_path)?;
    let keys = get_env_keys(&lines);
    
    for key in keys {
        println!("{key}");
    }
    
    Ok(())
}


fn randomize_command(
    file_path: &PathBuf, 
    keys: Vec<String>, 
    numeric: bool, 
    symbol: bool, 
    length: usize, 
    xkcd: bool
) -> Result<()> {
    let mut lines = read_env_file(file_path)?;
    
    for key in keys {
        let password = if xkcd {
            generate_xkcd_password()?
        } else {
            generate_random_password(length, numeric, symbol)
        };
        
        let mut found = false;
        for line in &mut lines {
            if let EnvLine::KeyValue { key: existing_key, value: existing_value } = line {
                if existing_key == &key {
                    *existing_value = password.clone();
                    found = true;
                    break;
                }
            }
        }
        
        if !found {
            lines.push(EnvLine::KeyValue { key, value: password });
        }
    }
    
    save_env_file(file_path, &lines)
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.