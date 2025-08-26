<p align="center">
  <img src="./branding/transparent-text.svg" alt="ChaiLauncher Logo" width="300" height="300">
</p>

# Modern Minecraft Launcher

*Inspired by GDLauncher, built with modern technologies*

<p align="">
  <a href="https://github.com/shield/ChaiLauncher">
    <img src="https://img.shields.io/badge/version-0.2.0-blue.svg" alt="Version">
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
  </a>
  <a href="https://tauri.app/">
    <img src="https://img.shields.io/badge/Tauri-2.8.2-orange.svg" alt="Tauri">
  </a>
  <a href="https://reactjs.org/">
    <img src="https://img.shields.io/badge/React-18.2.0-61dafb.svg" alt="React">
  </a>
</p>

## ✨ Features

- 🎮 **Modern Interface** - Clean, intuitive UI built with React and Tailwind CSS
- ⚡ **Fast Performance** - Powered by Tauri for native performance
- 🔐 **Microsoft OAuth** - Secure Minecraft account integration
- 📦 **Mod Management (WIP)** - Easy installation and management of mods
- 🌐 **Cross-Platform** - Supports Windows, macOS, and Linux
- 📊 **Real-time Progress** - Visual feedback for downloads and installations

## 🚀 Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) (v16 or later)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/tristanpoland/ChaiLauncher.git
   cd ChaiLauncher
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Run in development mode**
   ```bash
   npm run tauri:dev
   ```

4. **Build for production**
   ```bash
   npm run tauri:build
   ```

## 🛠️ Technology Stack

<div align="center">

| Frontend | Backend | Tools |
|----------|---------|-------|
| ![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB) | ![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white) | ![TypeScript](https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white) |
| ![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white) | ![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=Tauri&logoColor=white) | ![Vite](https://img.shields.io/badge/Vite-646CFF?style=for-the-badge&logo=vite&logoColor=white) |

</div>

### Core Dependencies

- **Frontend**: React 18, Tailwind CSS, Framer Motion, Radix UI
- **Backend**: Tauri 2.8, Tokio, Reqwest, OAuth2
- **Build Tools**: Vite, TypeScript, PostCSS

## 📱 Screenshots

![Home](./images/homescreen.png)

![Instances](./images/instances.png)

![accounts](./images/accounts.png)

![settings](./images/settings.png)


## 🤝 Contributing

We welcome contributions from the community! Here's how you can help:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add some amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Follow the existing code style and conventions
- Write clear, descriptive commit messages
- Test your changes thoroughly
- Update documentation as needed

## 📋 Roadmap

- [ ] Advanced mod management features
- [ ] Multiple Minecraft version support
- [ ] Custom modpack creation
- [ ] Server browser integration
- [ ] Shader pack management
- [ ] Resource pack management
- [ ] Performance optimization tools

## 🐛 Bug Reports & Feature Requests

Found a bug or have a feature request? Please open an issue on our [GitHub Issues](https://github.com/tristanpoland/ChaiLauncher/issues) page.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <p>Made with ❤️ by the ChaiLauncher Team</p>
  <p>
    <a href="https://github.com/tristanpoland/ChaiLauncher">⭐ Star this repo</a> |
    <a href="https://github.com/tristanpoland/ChaiLauncher/issues">🐛 Report Bug</a> |
    <a href="https://github.com/tristanpoland/ChaiLauncher/issues">💡 Request Feature</a>
  </p>
</div>
