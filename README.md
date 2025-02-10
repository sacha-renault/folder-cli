# folder-clip

> ⚠️ **Very Early Development Stage**: This project is in its initial development phase and is not yet ready for production use. Many features are either planned or partially implemented.
>
> ### Current Status
> - 🚧 Basic project structure setup
> - ✅ Core folder traversal implementation
> - 🚧 Basic tree visualization
> - ⏳ Filtering system (planned)
> - ⏳ Batch operations (planned)

A powerful command-line utility for advanced folder visualization.

## Features

- 🌳 Tree-like folder structure visualization
- 📋 Batch file operations with structure preservation
- 🔍 Advanced filtering using regex patterns
- 🚫 Exclusion patterns for unwanted files/folders
- 📁 Empty folder handling options
- 🎨 Customizable output formatting

## Installation

```bash
cargo install folder-clip
```

## Usage

### Display Folder Structure

```bash
# Basic folder structure display
folder-clip display ./my-project

# Exclude patterns with regex
folder-clip display ./my-project -e "node_modules|target|.git"

# Show empty folders
folder-clip display ./my-project --show-empty

# Custom depth limit
folder-clip display ./my-project --depth 3
```

## Options

### General Options

- `-e, --exclude <PATTERN>`: Regex pattern for files/folders to exclude
- `-i, --include <PATTERN>`: Regex pattern for files/folders to include
- `--show-empty`: Display empty folders
- `--depth <N>`: Maximum depth to traverse
- `-q, --quiet`: Suppress progress output
- `-v, --verbose`: Show detailed operation information

### Copy Options

- `--preserve`: Preserve file metadata (timestamps, permissions)
- `--overwrite`: Overwrite existing files
- `--dry-run`: Show what would be copied without actual copying

## Error Handling

The utility uses a custom error handling system that provides clear feedback:
- File permission issues
- Invalid regex patterns
- IO operations failures
- Filter-related exclusions

## Examples

### Complex Filtering

```bash
# Multiple exclusion patterns
folder-clip display . -e "\.git|target|node_modules" -e "\.tmp$|\.log$"

# Include only specific file types
folder-clip display . -i "\.rs$|\.toml$"
```

### Structure Preservation

```bash
# Copy only Rust project files while preserving structure
folder-clip copy ./project ./backup -i "\.rs$|\.toml$" --preserve
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development

```bash
# Clone the repository
git clone https://github.com/yourusername/folder-clip
cd folder-clip

# Build
cargo build

# Run tests
cargo test

# Run with example
cargo run -- display ./example-folder
```