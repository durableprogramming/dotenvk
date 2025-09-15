# dotenvk Demo

This directory contains demo scripts and VHS tapes for generating animated GIF demonstrations of `dotenvk` functionality.

## Files

- `demo.tape` - Full demonstration VHS script showcasing all major features
- `quickdemo.tape` - Quick demo VHS script for a shorter demonstration  
- `run_demo.sh` - Shell script to build dotenvk and run the full demo
- `quickdemo.sh` - Shell script to build dotenvk and run the quick demo

## Requirements

- [VHS](https://github.com/charmbracelet/vhs) - For recording terminal sessions as GIFs
- Rust/Cargo - For building the dotenvk binary

## Usage

### Full Demo

```bash
# Run the complete demo (generates demo.gif)
./run_demo.sh
```

### Quick Demo

```bash
# Run the quick demo (generates quickdemo.gif)
./quickdemo.sh
```

### Manual VHS Usage

```bash
# Build the project first
cargo build --release

# Add dotenvk to PATH temporarily
export PATH="../target/release:$PATH"

# Run VHS directly
vhs demo.tape        # Creates demo.gif
vhs quickdemo.tape   # Creates quickdemo.gif
```

## Demo Features

The full demo showcases:

- Setting environment variables with `dotenvk set`
- Generating secure passwords with `dotenvk randomize`
- Listing keys with `dotenvk keys`
- Exporting in bash and JSON formats with `dotenvk export`
- Removing variables with `dotenvk unset`
- File structure preservation and commenting

The quick demo provides a condensed version focusing on the core workflow.

## Output

- `demo.gif` - Full demonstration (approximately 30-40 seconds)
- `quickdemo.gif` - Quick demonstration (approximately 15-20 seconds)

Both GIFs are suitable for including in documentation, GitHub README files, or presentations.