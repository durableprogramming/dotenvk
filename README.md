# dotenvk

A pragmatic CLI tool for managing `.env` files with a focus on simplicity, security, and reliability.

## Overview

`dotenvk` provides a straightforward command-line interface for editing environment files. Built with Rust for performance and safety, it handles common tasks like setting variables, generating secure passwords, and exporting configurations.

![Demo](demo/demo.gif)

## Installation

```bash
cargo install dotenvk
```

Or build from source:

```bash
git clone https://github.com/durableprogramming/dotenvk.git
cd dotenvk
cargo build --release
```

## Usage

### Basic Commands

Set environment variables:
```bash
dotenvk set DATABASE_URL=postgres://localhost/mydb API_KEY=secret123
```

Remove variables:
```bash
dotenvk unset OLD_KEY DEPRECATED_VAR
```

List all keys:
```bash
dotenvk keys
```

### Password Generation

Generate secure random passwords:
```bash
# Basic 32-character password (letters only)
dotenvk randomize API_SECRET

# Include numbers and symbols
dotenvk randomize DB_PASSWORD --numeric --symbol --length 48

# Generate XKCD-style passphrase (requires xkcdpass)
dotenvk randomize ADMIN_PASSPHRASE --xkcd
```

### Export Formats

Export as bash script:
```bash
dotenvk export
# or
dotenvk export --format bash
```

Export as JSON:
```bash
dotenvk export --format json
```

### Working with Different Files

By default, `dotenvk` operates on `.env` in the current directory. Use `-f` or `--file` to specify a different file:

```bash
dotenvk -f .env.production set NODE_ENV=production
dotenvk -f config/.env.local keys
```

## Features

- **Preserves file structure**: Maintains comments, empty lines, and formatting
- **Safe updates**: Only modifies targeted key-value pairs
- **Secure passwords**: Generates cryptographically secure random passwords
- **Multiple export formats**: Bash and JSON output for integration with other tools
- **Simple interface**: Clear, predictable commands that do one thing well

## Philosophy

This tool embodies a practical approach to software development:

- **Solve real problems**: Managing environment files is a common task that deserves a dedicated tool
- **Keep it simple**: No complex configuration or unnecessary features
- **Be reliable**: Written in Rust for memory safety and consistent behavior
- **Respect existing work**: Preserves file formatting and comments
- **Security matters**: Provides secure password generation out of the box

## Examples

### Development Setup

```bash
# Set up a new project
dotenvk set NODE_ENV=development PORT=3000
dotenvk randomize JWT_SECRET SESSION_SECRET --length 64 --symbol

# Export for local development
eval $(dotenvk export)
```

### Production Deployment

```bash
# Generate secure production credentials
dotenvk -f .env.production randomize \
  DATABASE_PASSWORD \
  REDIS_PASSWORD \
  API_SECRET \
  --length 48 --numeric --symbol

# Export as JSON for container orchestration
dotenvk -f .env.production export --format json > secrets.json
```

### Password Rotation

```bash
# Rotate all password fields
dotenvk randomize $(dotenvk keys | grep -E '(PASSWORD|SECRET|KEY)')
```

## Contributing

Contributions are welcome. This project values:

- Clear, maintainable code
- Comprehensive testing
- Thoughtful documentation
- Backwards compatibility

Please ensure all tests pass and add new tests for any new functionality.

## License

MIT License - see LICENSE file for details.

## Acknowledgments

Built with excellent Rust crates:

- `clap` for CLI parsing
- `anyhow` for error handling
- `serde` for JSON serialization
- `rand` for secure random generation

