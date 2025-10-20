# Inkstone

A high-performance PDF rendering library and viewer written in Rust, supporting both native desktop and web applications.

## Overview

Inkstone is a comprehensive PDF rendering solution that provides:

- **Core PDF Library**: A robust PDF parsing and rendering engine
- **Native Desktop App**: A fast desktop PDF viewer built with [Iced](https://github.com/iced-rs/iced)
- **Web Application**: A browser-based PDF viewer built with [Dioxus](https://github.com/DioxusLabs/dioxus)

## Features

- 📄 **PDF Rendering**: High-quality PDF document rendering
- 🔤 **Font Support**: Comprehensive font handling with multiple encoding support
- 🎨 **Graphics**: Advanced graphics rendering with Pathfinder
- 🖥️ **Cross-Platform**: Works on desktop (Windows, macOS, Linux) and web browsers
- ⚡ **Performance**: Optimized rendering for smooth user experience

## Getting Started

### Prerequisites

- Rust 1.75+
- For web app: `wasm-pack` and `trunk`

### Building the Native App

```bash
cd native-app
cargo run --release
```

### Building the Web App

```bash
cd web-app
trunk serve
```

Visit `http://localhost:8080` in your browser.

### Using the Library

Add to your `Cargo.toml`:

```toml
[dependencies]
inkstone = { path = "path/to/inkstone" }
```

## Applications

### Native Desktop Viewer

A full-featured PDF viewer with:

- File browser integration
- Zoom and navigation controls
- High-performance rendering

### Web PDF Viewer

A modern web-based PDF viewer featuring:

- Drag-and-drop file loading
- Touch-friendly interface
- WebGL-accelerated rendering

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Sandip Dey** - [sandip.dey1988@yahoo.com](mailto:sandip.dey1988@yahoo.com)
