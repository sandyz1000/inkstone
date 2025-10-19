# Inkstone Web - PDF Viewer Web Application

A modern, high-performance PDF viewer built with Dioxus and WebAssembly.

## Features

- 🚀 **WebAssembly Performance** - Native-like speed in the browser
- 🎨 **Modern UI** - Clean, dark-themed interface with smooth animations
- 📱 **Responsive** - Works on desktop and mobile devices
- 🔍 **Zoom & Pan** - Multiple view modes and zoom levels
- 📑 **Page Navigation** - Thumbnail sidebar, page controls
- 🎯 **WebGL Rendering** - Hardware-accelerated PDF rendering using Pathfinder

## Building

### Prerequisites

- Rust 1.70+ with wasm32-unknown-unknown target
- [Trunk](https://trunkrs.dev/) - `cargo install trunk`
- [wasm-bindgen-cli](https://github.com/rustwasm/wasm-bindgen) - `cargo install wasm-bindgen-cli`

### Development

Run the development server with hot reload:

```bash
cd web-app
trunk serve --open
```

The app will be available at `http://localhost:8080`

### Production Build

Build an optimized release version:

```bash
cd web-app
trunk build --release
```

The output will be in the `dist/` directory.

## Project Structure

```
web-app/
├── src/
│   ├── app.rs              # Main application component and state
│   ├── components/         # UI components
│   │   ├── header.rs       # Top header with title and actions
│   │   ├── toolbar.rs      # Navigation and zoom controls
│   │   ├── pdf_canvas.rs   # WebGL canvas for PDF rendering
│   │   └── sidebar.rs      # Thumbnails and bookmarks sidebar
│   ├── pdf_viewer.rs       # PDF rendering logic with Pathfinder
│   ├── utils.rs            # WASM utilities and helpers
│   └── lib.rs              # Library entry point
├── index.html              # HTML template
├── Cargo.toml              # Dependencies
└── README.md               # This file
```

## Architecture

### Component Hierarchy

```
App
├── Header (file operations, title)
├── Main Content
│   ├── Sidebar (optional)
│   │   ├── Thumbnails
│   │   ├── Bookmarks
│   │   └── Attachments
│   └── Viewer Area
│       ├── Toolbar (navigation, zoom)
│       └── PDFCanvas (WebGL rendering)
```

### State Management

The application uses Dioxus signals for reactive state management:

- `AppState` - Global application state
  - Current page number
  - Total pages
  - Zoom level
  - View mode (single/continuous/two-page)
  - Sidebar visibility
  - File loaded status

### Rendering Pipeline

1. **PDF Loading** - Parse PDF with `pdf` crate
2. **Scene Generation** - Convert PDF page to Pathfinder scene
3. **WebGL Rendering** - Render scene with Pathfinder WebGL backend
4. **Canvas Display** - Display rendered result in HTML canvas

## Integration with Existing Code

The web app integrates with the existing inkstone codebase:

- **pdf_view** - PDF document abstraction and page rendering
- **pdf_render** - PDF-to-scene conversion with `SceneBackend`
- **pathfinder** - Hardware-accelerated vector graphics rendering

## Keyboard Shortcuts

- `←/→` or `PgUp/PgDn` - Navigate pages
- `Shift + ←/→` - Jump 10 pages
- `Ctrl + 0` - Reset zoom to 100%
- `Ctrl + +/-` - Zoom in/out

## Browser Support

- Chrome/Edge 90+
- Firefox 89+
- Safari 15+

WebGL 2.0 support is required.

## Development Tips

### Debugging

Enable debug logging in the browser console:

```javascript
localStorage.setItem('RUST_LOG', 'info');
```

### Performance

The release build is highly optimized:
- LTO enabled
- Small code size (`opt-level = "z"`)
- Single codegen unit

Typical WASM bundle size: ~500KB (gzipped)

## TODO

- [ ] File upload/drag-and-drop support
- [ ] Text selection and search
- [ ] Annotations and highlights
- [ ] Bookmarks management
- [ ] Print functionality
- [ ] Download/save as PDF
- [ ] Full-screen mode
- [ ] Presentation mode
- [ ] Dark/light theme toggle
- [ ] Mobile touch gestures
- [ ] Keyboard navigation

## License

MIT
