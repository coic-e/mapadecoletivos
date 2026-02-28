# Mapadecoletivos Monorepo

A monorepo containing the Mapadecoletivos platform - mapping collectives and community organizations.

## 📁 Project Structure

```
mapadecoletivos/
├── mapadecoletivos-api/          # Node.js/TypeScript API (submodule)
├── mapadecoletivos-app/          # React frontend application (submodule)
├── mapadecoletivos-api-rust/     # Rust API implementation (submodule)
├── Cargo.toml                    # Rust workspace configuration
└── git-commands.sh               # Git submodule management helper
```

## 🚀 Getting Started

### Prerequisites

- **Node.js** (v14+) and npm/yarn
- **Rust** (latest stable) and Cargo
- **Git**

### Initial Setup

Clone the repository with all submodules:

```bash
git clone --recursive git@github.com:coic-e/mapadecoletivos.git
cd mapadecoletivos
```

Or if already cloned, initialize submodules:

```bash
./git-commands.sh setup
```

## 🛠️ Development

### Rust Workspace

This is a Cargo workspace for Rust projects. From the root directory:

```bash
# Build all Rust projects
cargo build

# Run tests
cargo test

# Run a specific project
cargo run -p mapadecoletivos-api-rust

# Check code without building
cargo check

# Add a dependency to a specific project
cargo add serde -p mapadecoletivos-api-rust
```

### Node.js Projects

#### API (TypeScript)
```bash
cd mapadecoletivos-api
npm install
npm run dev
```

#### Frontend App (React)
```bash
cd mapadecoletivos-app
npm install
npm start
```

## 📦 Submodule Management

Use the provided `git-commands.sh` script to manage submodules:

```bash
# Initialize and setup all submodules
./git-commands.sh setup

# Update all submodules to main/master branch
./git-commands.sh update-main

# Update all submodules to develop branch
./git-commands.sh update-develop

# Show help
./git-commands.sh help
```

### Manual Submodule Commands

```bash
# Update all submodules to latest
git submodule update --remote --recursive

# Pull latest changes in all submodules
git submodule foreach git pull

# Check status of all submodules
git submodule status
```

## 🏗️ Architecture

### mapadecoletivos-api (Node.js)
- TypeScript backend API
- TypeORM for database management
- Express/similar framework
- PostgreSQL database

### mapadecoletivos-app (React)
- React frontend application
- TypeScript
- Leaflet for map visualization
- Responsive design

### mapadecoletivos-api-rust
- Rust implementation of the API
- Actix-web framework
- Modern async runtime
- High-performance alternative to Node.js API

## 🔗 Repository Links

- Main: [mapadecoletivos](https://github.com/coic-e/mapadecoletivos)
- API: [mapadecoletivos-api](https://github.com/coic-e/mapadecoletivos-api)
- App: [mapadecoletivos-app](https://github.com/coic-e/mapadecoletivos-app)
- Rust API: [mapadecoletivos-api-rust](https://github.com/coic-e/mapadecoletivos-api-rust)

## 📝 Contributing

When working with submodules:

1. Make changes in the respective submodule directory
2. Commit and push in the submodule repository
3. Return to the main repository and commit the submodule reference update

```bash
# Example workflow
cd mapadecoletivos-api-rust
git checkout -b feature/new-endpoint
# ... make changes ...
git commit -m "Add new endpoint"
git push origin feature/new-endpoint

cd ..
git add mapadecoletivos-api-rust
git commit -m "Update rust api submodule"
git push
```

## 📄 License

See individual project repositories for license information.
