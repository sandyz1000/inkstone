# Inkstone Native PDF Viewer

A native PDF viewer application built with GPUI (from gpui-component framework).

## Features

- **PDF Rendering**: High-quality PDF rendering using the inkstone rendering engine
- **Page Navigation**: Navigate through PDF pages using arrow keys or buttons
- **Zoom Controls**: Zoom in/out with keyboard shortcuts or buttons
- **Modern UI**: Clean, dark-themed interface built with GPUI
- **Keyboard Shortcuts**: Full keyboard control for efficient navigation

## Keyboard Shortcuts

- `Cmd+O` - Open PDF file
- `←` / `→` - Previous/Next page
- `+` / `-` - Zoom in/out
- `Cmd+0` - Reset zoom to 100%

## Building

```bash
cd native-app
cargo build --release
```

## Running

```bash
cargo run --release
```

The application will launch with an empty window. Click "Open PDF" or press `Cmd+O` to select a PDF file to view.

## Dependencies

- **gpui-component**: Modern UI framework for building native applications
- **inkrender**: PDF rendering engine
- **pathfinder**: Vector graphics rendering
- **rfd**: Native file dialog

## Architecture

The application consists of three main components:

1. **main.rs**: Application entry point and window setup
2. **app.rs**: Main application state and UI rendering logic
3. **renderer.rs**: PDF rendering and page management

The app uses GPUI's reactive rendering system to update the UI when the PDF state changes (page navigation, zoom, etc.).

## Migration from Iced

This application was previously built with the Iced UI framework. It has been migrated to GPUI for:
- Better integration with the gpui-component ecosystem
- More flexible layout system
- Native macOS integration
- Improved performance

## Notes

- The application renders PDF pages to PNG images temporarily and displays them
- OpenGL rendering is done in a separate thread to avoid conflicts
- The `STANDARD_FONTS` environment variable can be set to point to a custom fonts directory
