# ZippyPack Architecture

**Created by: Kamion (Matthéo Le Fur)**  
**Date: July 14, 2025**  
**Version: 1.0.0**

## Overview

ZippyPack is architected around several specialized modules that collaborate to provide advanced compression with block-level deduplication.

## Module Structure

### 🏗️ Core Modules

#### `src/main.rs`
- **Role**: Main CLI interface
- **Responsibilities**: Argument parsing, command dispatch
- **Dependencies**: clap, env_logger

#### `src/lib.rs`
- **Role**: Public module exposure
- **Responsibilities**: Export organization, tests

#### `src/compress.rs`
- **Role**: Traditional compression (.zpp)
- **Responsibilities**: Folder compression, type detection
- **Algorithms**: zstd, solid compression

#### `src/decompress.rs`
- **Role**: .zpp archive decompression
- **Responsibilities**: File restoration, integrity validation
- **Security**: Path sanitization

#### `src/image.rs` 🚀
- **Role**: Image system with deduplication
- **Responsibilities**: .zpak image creation/extraction
- **Innovation**: 64KB block-level deduplication

#### `src/profile.rs`
- **Role**: Type-specific compression profiles
- **Responsibilities**: Contextual optimization
- **Supported Types**: Text, Binary, GameEngine, etc.

#### `src/error.rs`
- **Role**: Typed error handling
- **Responsibilities**: Specific error definitions

## Data Flow

### Traditional Compression
```
Folder → Scan files → Type detection → Compression → .zpp
```

### Image System
```
Folder → Scan files → Block splitting → Deduplication → Index → .zpak
```

### Decompression
```
.zpp/.zpak → Read index → Decompress blocks → Restore files
```

## File Formats

### .zpp Format (Traditional Compression)
1. **Header**: Dictionary size (8 bytes)
2. **Dictionary**: zstd dictionary data
3. **Compressed Data**: Solid zstd stream

### .zpak Format (Image System)
1. **Header**: Version, stats, metadata (48 bytes)
2. **Block Index**: Hash + position + size of each block
3. **Compressed Data**: Deduplicated zstd blocks
4. **File Metadata**: Directory tree + block references

## Key Algorithms

### Block-Level Deduplication
- **Block Size**: 64KB (65536 bytes)
- **Hash**: DefaultHasher (simple but efficient)
- **Storage**: HashMap<BlockHash, DataBlock>

### zstd Compression
- **Levels**: 1-22 (default: 22)
- **Solid Mode**: Available for .zpp
- **Dictionaries**: Automatic generation

## Performance

### Time Complexity
- **Compression**: O(n) where n = total size
- **Deduplication**: O(n/64KB) for indexing
- **Decompression**: O(n) linear

### Space Complexity
- **Memory**: O(number of unique blocks)
- **Storage**: O(unique data after deduplication)

## Extensibility

### Adding New Formats
1. Create new module in `src/`
2. Define Options structures
3. Implement create/extract functions
4. Add CLI commands in `main.rs`

### New Algorithms
1. Modify `compress.rs` for integration
2. Add profiles in `profile.rs`
3. Update tests

## Security

### Sanitization
- **Paths**: Windows/Unix character validation
- **Size**: Limits on blocks and files
- **Integrity**: Checksums on critical data

### Mitigated Vulnerabilities
- **Path traversal**: Relative path cleaning
- **Zip bombs**: Decompression limits
- **Memory exhaustion**: Large file streaming

## Testing

### Test Structure
- `src/tests/compression_tests.rs`: Unit tests
- `examples/`: Usage examples
- `tools/`: Test utilities

### Coverage
- ✅ Basic compression/decompression
- ✅ Image system
- ✅ Round-trip integrity
- ✅ Error handling

## 🌍 Translations

- [Français (French)](ARCHITECTURE_FR.md)

---

This architecture enables modular evolution while maintaining optimal performance and robust security.