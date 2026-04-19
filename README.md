# Git Gud

A lightweight, high-performance Git GUI built with **Rust**, **Tauri**, and **Vanilla JavaScript**. Inspired by tools like GitFork, **Git Gud** aims to provide a fast and intuitive experience for managing your Git repositories.

## ✏️ Author Note

This project is 99% AI generated for me to better understand AI code generation workflows and model capacities. The intention for the future is to make a usable git GUI client but for now is lacking too many features to be used in a real-life scenario. 

## ✨ Features

- **📂 Multi-Repo Tabs**: Open and manage multiple repositories simultaneously in a clean tabbed interface.
- **🔄 Session Persistence**: Remembers your open repositories and window dimensions across sessions.
- **⚡ Real-time Status**: Monitors your filesystem and automatically updates the UI when files are modified, staged, or committed (even from outside the app).
- **📝 Stage & Commit**: Easily stage/unstage individual files or everything at once. Supports the standard subject/body commit format.
- **🔧 Amend Commit**: One-click to amend your last commit, with automatic message pre-filling.
- **🌿 Branch Management**: Rename local branches via a clean right-click context menu.
- **📂 Native Dialogs**: Uses system-native folder pickers for a seamless OS experience.

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/git-gud.git
   cd git-gud
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

4. Build for production:
   ```bash
   npm run tauri build
   ```

## 🛠️ Tech Stack

- **Backend**: Rust, [git2-rs](https://github.com/rust-lang/git2-rs), [notify](https://github.com/notify-rs/notify)
- **Frontend**: Vanilla JavaScript, CSS3 (Flexbox/Grid), HTML5
- **Framework**: [Tauri v2](https://tauri.app/)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
