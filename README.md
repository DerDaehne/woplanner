# WOPlanner 🏋️

A Progressive Web App (PWA) for tracking workouts and monitoring strength progression. Built with Rust, HTMX, and focused on simplicity and performance.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![SQLite](https://img.shields.io/badge/sqlite-%2307405e.svg?style=for-the-badge&logo=sqlite&logoColor=white)
![HTMX](https://img.shields.io/badge/htmx-%23FF0000.svg?style=for-the-badge&logo=htmx&logoColor=white)

## Features ✨

### Core Functionality
- **User Management** - Multi-user support with session-based authentication
- **Exercise Library** - Create and manage custom exercises with instructions
- **Workout Planning** - Build structured workout routines with multiple exercises
- **Live Training** - Guided workout experience with real-time set tracking
- **Training History** - Review past workouts and track progression over time
- **Smart Scheduling** - Manual, weekly, or rotation-based workout scheduling

### User Experience
- **Mobile-First Design** - Optimized for iOS and Android PWA
- **Glassmorphism UI** - Modern, dark theme
- **Single Exercise Focus** - Guided flow through exercises and sets
- **Real-time Progress** - See your workout completion percentage
- **Touch-Friendly** - Large tap targets and smooth animations


## Tech Stack 🛠️

### Backend
- **[Rust](https://www.rust-lang.org/)**
- **[Axum](https://github.com/tokio-rs/axum)**
- **[SQLx](https://github.com/launchbadge/sqlx)**
- **[Tower Sessions](https://github.com/maxcountryman/tower-sessions)**

### Frontend
- **[HTMX](https://htmx.org/)**
- **[TailwindCSS](https://tailwindcss.com/)**
- **[Askama](https://github.com/djc/askama)**

### Infrastructure
- **[Nix](https://nixos.org/)**
- **SQLite**

## Getting Started 🚀

### Prerequisites

- **Nix** (recommended) - For reproducible dev environment (not _really_ necessary though)
- **Rust** 1.70+ - [Install via rustup](https://rustup.rs/)
- **SQLite** - Usually pre-installed on Unix systems

### Development Setup

#### Option 1: Using Nix (Recommended)

```bash
# clone the repository
git clone https://github.com/DerDaehne/woplanner.git
cd woplanner

# enter Nix development shell
nix develop

# run the application
cargo run
```

#### Option 2: Using Cargo

```bash
# clone the repository
git clone https://github.com/DerDaehne/woplanner.git
cd woplanner

# set database URL (optional, defaults to ./bodybuilding.db)
export DATABASE_URL="sqlite:./bodybuilding.db"

# run database migrations (automatic on first run)
cargo run
```

### Access the application

open your browser and navigate to:
```
http://localhost:3000
```

### Development Workflow

```bash
# auto-restart on code changes
cargo watch -x run

# run tests (no tests implemented yet!)
cargo test

# run linter
cargo clippy

# format code
cargo fmt

# build for production
cargo build --release
```

## Configuration ⚙️

### Environment Variables

```bash
# database location (default: sqlite:./bodybuilding.db)
DATABASE_URL="sqlite:./path/to/db.db"

# server port (default: 3000)
PORT=3000
```
## Roadmap 🗺️

### In Progress
- [x] user management
- [x] exercise library
- [x] workout planning
- [x] live training tracking
- [x] basic dashboard
- [ ] training history (WIP)

### Planned Features
- [ ] exercise progression charts
- [ ] body measurements tracking
- [ ] rest timer between sets
- [ ] personal Records (PRs) detection
- [ ] training streaks & achievements
- [ ] workout templates
- [ ] progressive overload suggestions
- [ ] full offline support
- [ ] export/import data

## License 📄

This project is open source and available under the [MIT License](LICENSE).


**Note:** This is a learning project focused on Rust, HTMX, and PWA development. Features are continuously being added and improved. I'm doing this all in my spare time for myself.
