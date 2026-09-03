# WinOptimizer - Windows Optimization Desktop App

> **WinOptimizer** is a comprehensive Windows optimization desktop application built with **Rust + Tauri** and integrated with **FreeLLMAPI** for intelligent automation and optimization tasks.

## 🎯 Overview

WinOptimizer provides a complete suite of Windows system optimization tools, including:

- **Debloat** - Remove bloatware, telemetry, and unnecessary Windows services
- **Gaming Mode** - Optimize system performance for gaming (GPU scheduling, network tweaks, etc.)
- **Memory Cleaner** - Clean RAM and optimize memory usage
- **Services & Startup** - Manage Windows services and startup applications
- **Disk Cleaner & Duplicate Finder** - Clean temporary files and detect duplicate files
- **Diagnostics & Repair** - System diagnostics and repair tools
- **Advanced Suite** - GPU optimization, privacy hardening, Windows Update management
- **One-Click Optimization** - One-button optimization combining key optimization tasks
- **In-App Update Checker** - Automatic GitHub release update detection

## 🚀 Quick Start

### Installation

#### Windows
1. Download the latest release:
   ```bash
   # Visit: https,github.com/duongmeowww/win-optimizer/releases
   # Download: WinOptimizer_0.4.2_x64-setup.exe
   ```

2. Chạy file installer:
   ```
   WinOptimizer_0.4.2_x64-setup.exe
   ```

3. Khởi động WinOptimizer (sau khi cài đặt).

#### From Source

1. Clone repository:
   ```bash
   git clone https://github.com/duongmeogww/win-optimizer.git
   cd win-optimizer
   ```

2. Build frontend:
   ```bash
   npm run build
   ```

3. Build backend:
   ```bash
   cargo build --release
   ```

4. Khởi động Tauri:
   ```bash
   npm run tauri build && src-tauri/target/release/win-optimizer.exe
   ```

## 🛠️ Usage

### Navigation
- **Dashboard** - System overview, health score, quick optimization
- **Debloat** - Remove bloatware and telemetry
- **Gaming Mode** - Optimize for gaming performance
- **Memory Cleaner** - Clean RAM and optimize memory
- **Services & Startup** - Manage system services
- **Disk Cleaner & Duplicate Finder** - Clean disk space
- **Diagnostics & Repair** - System diagnostics and repair
- **Advanced Suite** - Advanced optimization tools
- **Tinh chỉnh & Công cụ** - Registry tweaks, advanced tools

### Features

#### One-Click Optimization
The **One-Click Optimization** button on the Dashboard performs the following tasks:

1. **Memory Cleanup** - Clean RAM and optimize memory usage
2. **Safe Debloat** - Remove common bloatware (Spotify, TikTok, Teams, etc.)
3. **Safe Gaming Tweaks** - Apply gaming optimizations (Game Mode, Game DVR, etc.)

#### Update Checker
Automatically checks GitHub releases for updates and notifies you when a new version is available.

## 🎨 Features Details

### Debloat
Removes Windows bloatware applications and telemetry services.

### Gaming Mode
Optimizes system settings for gaming performance, including:
- Hardware-accelerated GPU scheduling
- Network optimizations for gaming
- Game mode settings

### Memory Cleaner
Cleans RAM and optimizes memory usage by:
- Clearing memory pools
- Terminating memory-heavy processes
- Optimizing memory allocation

### Services & Startup
Manages Windows services and startup applications to:
- Disable unnecessary services
- Optimize startup performance
- Improve system responsiveness

### Disk Cleaner & Duplicate Finder
Cleans disk space by removing:
- Temporary files
- Cache files
- Duplicate files

### Diagnostics & Repair
Runs system diagnostics and repairs:
- System file checks (SFC, DISM)
- Disk health checks (S.M.A.R.T)
- Event log analysis

### Advanced Suite
Advanced optimization tools:
- GPU optimization
- Privacy hardening
- Windows Update management

### Tinh chỉnh & Công cụ
- Registry tweaks for optimization
- Advanced tool execution
- System settings management

## 🛠 Technical Details

### Built With
- **Frontend**: React + TypeScript + Ant Design
- **Backend**: Rust + Tauri + Electron
- **Package Manager**: npm (frontend), cargo (backend)
- **Build Tools**: Vite, Tauri CLI
- **API**: Tauri invoke IPC

### Key Features
- **Cross-platform**: Currently Windows only
- **Auto-update**: Checks GitHub releases automatically
- **Lightweight**: Optimized for performance
- **Secure**: Uses native Windows APIs
- **Extensible**: Plugin architecture for future features

## 📋 System Requirements

### Minimum Requirements
- **OS**: Windows 10 or later (64-bit)
- **CPU**: x86_64 (64-bit processor)
- **RAM**: 4 GB minimum
- **Disk Space**: 500 MB free space
- **Internet**: For update checks and FreeLLMAPI

### Recommended Requirements
- **OS**: Windows 11
- **CPU**: Modern x86_64 processor
- **RAM**: 8 GB or more
- **Disk Space**: 1 GB free space
- **GPU**: DirectX 12 compatible graphics card

## 📸 Screenshots

### Dashboard
![Dashboard](screenshots/dashboard.png)

### Debloat
![Debloat](screenshots/debloat.png)

### Gaming Mode
![Gaming Mode](screensscreens/gaming.png)

### Memory Cleaner
![Memory Cleaner](screenshots/memory-cleaner.png)

## �Licensing

This project is licensed under the MIT License - see the LICENSE file for details.

## Contact

For support or questions, please visit the GitHub repository or contact the maintainer.

## Acknowledgment

- This project was inspired by various Windows optimization tools and community contributions.
- Special thanks to the Rust and Tauri communities for their excellent tooling.
- Thanks to all contributors who have helped make this project better.

## Changelog

### Version 0.4.2
- Added One-Click Optimization feature on Dashboard
- Fixed HelpButton modal text color (white-on-white)
- Improved update checker with PowerShell-based GitHub API call
- Added comprehensive error handling for update checks
- Enhanced UI/UX for better user experience

### Version 0.4.1
- Added App Manager feature
- Enhanced settings and configuration options
- Improved performance and stability

### Version 0.4.0
- Initial release with core optimization features
- Basic UI framework and components

## Contribute

We welcome contributions to this project! Please see our contribution guidelines for more information.

## Acknowledgements

- **Tauri** - Cross-platform application framework
- **Rust** - Systems programming language
- **React** - JavaScript library for building user interfaces
- **Ant Design** - Design system for React
- **FreeLLMAPI** - AI model API integration
- **GitHub Actions** - CI/CD automation

## Support

If you encounter any issues or have questions, please create an issue on the GitHub repository.

## Thank You

Thank you for using WinOptimizer! We hope this tool helps you optimize your Windows system effectively and efficiently.

---

*Last updated: September 3, 2026*