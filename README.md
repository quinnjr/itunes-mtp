# iTunes to MTP Sync

A desktop application built with [Tauri](https://tauri.app/) that synchronizes your iTunes library playlists to MTP (Media Transfer Protocol) devices. Transfer your favorite playlists from iTunes to your phone, MP3 player, or other MTP-compatible devices with ease.

## 🎵 Overview

This application allows you to:
- Parse iTunes library XML files to extract playlists and track metadata
- Connect to MTP devices (Android phones, MP3 players, etc.)
- Synchronize selected playlists to your device with proper folder organization
- Track sync progress and handle errors gracefully

## ✨ Features

### Current (MVP)
- ✅ Basic iTunes library XML parsing
- ✅ MTP device detection and enumeration
- ✅ Device connection management
- ✅ Basic UI for library upload and playlist selection

### Planned
- 📋 Complete playlist parsing from iTunes XML
- 📁 Automatic folder structure creation on device (Artist/Album organization)
- 📤 File upload to MTP devices
- 🔄 End-to-end playlist synchronization
- 📊 Real-time sync progress tracking
- 🔍 Duplicate detection and handling
- ⚙️ Configurable sync options
- 🎨 Polished user interface

See [ROADMAP.md](./ROADMAP.md) for detailed development phases and [TODO.md](./TODO.md) for the complete task list.

## 🛠️ Tech Stack

### Frontend
- **Framework**: [Angular](https://angular.io/) 20 (Standalone components, Signals)
- **UI Library**: [PrimeNG](https://primeng.org/) v19
- **Styling**: [TailwindCSS](https://tailwindcss.com/) v4
- **Testing**: [Vitest](https://vitest.dev/) with jsdom

### Backend
- **Language**: [Rust](https://www.rust-lang.org/)
- **Framework**: [Tauri](https://tauri.app/) v2
- **MTP Library**: Windows Portable Device API (via COM)

### Development Tools
- **Package Manager**: [pnpm](https://pnpm.io/)
- **Linting**: ESLint with Angular rules
- **Git Hooks**: [Husky](https://typicode.github.io/husky/) with Commitlint
- **Module System**: ESM-only (`.mjs` files)

## 📋 Prerequisites

- **Node.js** (check `.nvmrc` or `package.json` engines)
- **pnpm** package manager ([install guide](https://pnpm.io/installation))
- **Rust** toolchain (for Tauri backend)
  ```bash
  # Install Rust
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  
  # Add clippy for linting (optional but recommended)
  rustup component add clippy
  ```
- **Git** (for version control)

## 🚀 Installation

1. **Clone the repository**
   ```bash
   git clone git@github.com:quinnjr/itunes-mtp.git
   cd itunes-mtp
   ```

2. **Install dependencies**
   ```bash
   pnpm install
   ```

3. **Verify installation**
   ```bash
   # Run linting
   pnpm lint
   
   # Run tests
   pnpm test
   
   # Build the application
   pnpm build
   ```

## 💻 Usage

### Development

Start the development server:

```bash
pnpm start
```

This will launch the Angular development server. The Tauri app will open automatically.

### Building

Build the application for production:

```bash
# Build web assets
pnpm build

# Build Tauri application
pnpm tauri build
```

The compiled application will be in `src-tauri/target/release/`.

### Testing

```bash
# Run all tests
pnpm test

# Run tests in watch mode
pnpm test --watch

# Run linting
pnpm lint

# Auto-fix linting issues
pnpm lint:fix
```

## 🔧 Development

### Project Structure

```
itunes-mtp/
├── src/                    # Angular frontend
│   ├── app/               # Application components and services
│   └── assets/            # Static assets
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   └── Cargo.toml         # Rust dependencies
├── .cursor/               # Cursor AI rules
│   └── rules/            # Coding standards and rules
├── .husky/                # Git hooks (Husky)
└── package.json           # Node.js dependencies and scripts
```

### Code Standards

This project follows strict coding standards enforced by Cursor AI rules and automated checks:

- **TypeScript/Angular**: See [`.cursor/rules/angular-typescript.mdc`](.cursor/rules/angular-typescript.mdc)
- **Rust**: See [`.cursor/rules/rust.mdc`](.cursor/rules/rust.mdc)
- **Testing**: See [`.cursor/rules/testing.mdc`](.cursor/rules/testing.mdc)
- **JavaScript Files**: All JS config files use `.mjs` extension (ESM-only)
- **Package Management**: Always use `pnpm` instead of `npm` or `yarn`

### Git Workflow

This project uses **Git Flow** for branch management. See [CONTRIBUTORS.md](./CONTRIBUTORS.md) for detailed workflow instructions.

**Quick Start:**
```bash
# Create a feature branch
git checkout development
git pull origin development
git checkout -b feature/my-feature-name

# Make changes and commit (with conventional commit message)
git add .
git commit -m "feat(scope): add new feature"

# Push and create merge request
git push origin feature/my-feature-name
```

**Important:**
- Direct commits to `main` or `development` are blocked
- All commits must follow [Conventional Commits](https://www.conventionalcommits.org/) format
- Pre-commit hooks run linting and tests automatically
- Commit messages are validated by commitlint

### Pre-commit Hooks

Before every commit, the following checks run automatically:
- ✅ ESLint (TypeScript/Angular)
- ✅ Vitest tests
- ✅ Rust clippy (if installed)

All checks must pass before the commit is accepted.

## 📚 Documentation

- **[CONTRIBUTORS.md](./CONTRIBUTORS.md)** - Contribution guidelines and Git Flow workflow
- **[ROADMAP.md](./ROADMAP.md)** - Development phases and timeline
- **[TODO.md](./TODO.md)** - Current tasks and feature list
- **[LICENSE.md](./LICENSE.md)** - MIT License

## 🐛 Troubleshooting

### Common Issues

**Git hooks not working:**
```bash
# Ensure Husky is installed
pnpm install

# Verify hooks are executable (Unix-like systems)
chmod +x .husky/*
```

**Tests failing:**
- Ensure all dependencies are installed: `pnpm install`
- Check that Vitest is configured correctly (see `vitest.config.mjs`)

**Rust clippy not found:**
```bash
# Install Rust clippy
rustup component add clippy
```

**MTP device not detected:**
- Ensure device is connected and unlocked
- Check that Windows recognizes the device
- Try disconnecting and reconnecting the device

## 🤝 Contributing

We welcome contributions! Please read [CONTRIBUTORS.md](./CONTRIBUTORS.md) for:
- Git Flow workflow
- Commit message conventions
- Code standards
- Testing requirements

## 📄 License

This project is licensed under the MIT License - see [LICENSE.md](./LICENSE.md) for details.

Copyright © 2024 Joseph R. Quinn

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) for the excellent desktop app framework
- [Angular](https://angular.io/) for the robust frontend framework
- [PrimeNG](https://primeng.org/) for the UI component library
- [Vitest](https://vitest.dev/) for fast testing

---

**Status**: 🚧 In Active Development

See [ROADMAP.md](./ROADMAP.md) for current development status and planned features.

