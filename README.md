# ZippyPack

**ZippyPack** is an advanced Rust compression tool that leverages Zstandard algorithm with block-level deduplication and system image format for superior compression ratios.

## 🚀 Features

- **Zstd Compression**: Modern Zstandard algorithm for optimal speed/ratio balance
- **Block Deduplication**: Store identical data blocks only once (64KB chunks)
- **System Image Format**: Complete folder snapshots with instant access
- **Context-Aware Compression**: File-type specific optimizations
- **Real-time Progress**: Detailed progress with speed and ETA
- **Cross-platform**: Compatible with Linux, macOS, and Windows

## 📊 Performance

On a dataset of 505 source code files:
- **Compression Ratio**: 95.67% (5.1 MB → 222 KB)
- **Comparison**: 6% gap with WinRAR, 12% better than 7-Zip
- **Speed**: ~0.2 MB/s with maximum compression

## 📁 Project Structure

```
zippypack/
├── src/                    # Main source code
│   ├── main.rs            # CLI interface
│   ├── lib.rs             # Public library
│   ├── compress.rs        # Traditional compression
│   ├── decompress.rs      # Decompression
│   ├── image.rs           # Image system with deduplication
│   ├── profile.rs         # Compression profiles
│   └── error.rs           # Error handling
├── examples/              # Usage examples
├── tools/                 # Development utilities
├── docs/                  # Technical documentation
└── README.md             # This file
```

## 🔧 Installation

```bash
git clone https://github.com/kamionn/zippypack.git
cd zippypack
cargo build --release
```

## 📖 Usage

### Classic Compression (.zpp)
```bash
# Compress a folder
cargo run --release -- compress --input folder/ --output archive.zpp --level 22

# Decompress an archive
cargo run --release -- decompress --input archive.zpp --output restored_folder/
```

### System Image (.zpak)
```bash
# Create system image with deduplication
cargo run --release -- create-image --input project/ --output backup.zpak --level 22

# Extract system image
cargo run --release -- extract-image --input backup.zpak --output restored_project/
```

### Advanced Options
```bash
# Compression with custom threads
cargo run --release -- compress --input src/ --output code.zpp --threads 8 --level 15

# Solid mode for better compression
cargo run --release -- compress --input data/ --output data.zpp --solid --level 22
```

## 🏗️ Architecture

### Core Modules
- **`compress.rs`**: Traditional compression with type detection
- **`decompress.rs`**: Decompression with integrity validation
- **`image.rs`**: Image system with block-level deduplication
- **`profile.rs`**: File-type compression profiles
- **`error.rs`**: Typed error handling

### Archive Format (.zpak)
1. **Header**: Version, metadata, statistics
2. **Block Index**: Hash and position of each unique block
3. **Compressed Data**: Deduplicated zstd blocks
4. **File Metadata**: Directory tree and block references

## 🧪 Testing

```bash
# Unit tests (in modules)
cargo test

# Verbose tests
cargo test -- --nocapture

# Usage example
cargo run --bin basic_usage
```

## 📈 Advantages vs Competition

| Feature | ZippyPack | WinRAR | 7-Zip |
|---------|-----------|--------|-------|
| Deduplication | ✅ | ❌ | ❌ |
| Instant Access | ✅ | ❌ | ❌ |
| Real-time Progress | ✅ | ❌ | ❌ |
| Modern Format | ✅ | ❌ | ❌ |
| Cross-platform | ✅ | ❌ | ✅ |

## 🔬 Optimal Use Cases

- **Development Projects**: node_modules, target/, build/
- **Incremental Backups**: Massive deduplication benefits
- **Game Assets**: Similar textures and models
- **Documentation Archives**: Files with repetitive patterns

## 🛣️ Roadmap

- [ ] Incremental compression
- [ ] FUSE mounting for direct access
- [ ] Graphical interface
- [ ] CI/CD integration
- [ ] Optimized cloud synchronization

## 🤝 Contributing

Contributions are welcome! Check out the [issues](https://github.com/Kamionn/zippypack/issues) for ongoing tasks.

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 🏆 Benchmarks

```bash
# Generate test files
rustc tools/generate_test_files.rs && ./generate_test_files

# Test compression
cargo run --release -- create-image --input test_files --output benchmark.zpak --level 22

# Compare with other tools
# WinRAR: 268 KB
# 7-Zip: 324 KB  
# ZippyPack: 284 KB
```

## 🌍 Translations

- [Français (French)](README_FR.md)

---

**ZippyPack**: Because every byte counts. 🚀
