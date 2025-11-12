# My GPUI App

A modern Windows desktop application built with Rust and GPUI framework.

## Features

- 🎨 Modern UI with GPUI Component library
- 🚀 High-performance native rendering
- 🎯 Stateful component management
- 🌈 Theme system support
- 📱 Responsive layouts

## Project Structure

```
my-gpui-app/
├── .cargo/
│   └── config.toml        # Windows stack size configuration
├── src/
│   ├── main.rs            # Application entry point
│   ├── app.rs             # Main application logic
│   ├── views/             # UI views
│   │   ├── home.rs        # Home page view
│   │   └── settings.rs    # Settings page view
│   ├── state/             # State management
│   │   └── app_state.rs   # Global app state
│   ├── components/        # Custom components
│   └── utils/             # Utility functions
└── Cargo.toml             # Project configuration
```

## Prerequisites

- Rust (latest stable)
- Windows 10 or later
- Visual Studio Build Tools (for MSVC toolchain)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run
```

## Development

The application includes:

- **Home Page**: Welcome screen with a counter demo
- **Settings Page**: Configuration and preferences
- **Sidebar Navigation**: Easy navigation between pages

## Architecture

### Entry Layer (main.rs)
- Initializes GPUI Application
- Creates main window with Root component

### Application Layer (app.rs)
- Implements Render trait
- Manages global state
- Handles routing and page switching

### View Layer (views/)
- Independent view modules for each page
- Uses stateless RenderOnce elements
- Handles user interactions

### State Management (state/)
- Uses GPUI's Entity<T> for stateful components
- Manages application-wide state

## Technologies

- **GPUI**: High-performance UI framework
- **GPUI Component**: Rich component library
- **Rust**: Systems programming language

## License

MIT
# bilibili-agent
