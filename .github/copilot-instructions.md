# Mapadecoletivos AI Coding Guide

## Project Architecture

This is a **Git submodules monorepo** (not Lerna/Turborepo) containing three independent services for mapping electronic music collectives in Brazil:

- **mapadecoletivos-api/** - TypeScript/Node.js REST API (Express + TypeORM + PostgreSQL)
- **mapadecoletivos-api-rust/** - Rust API alternative (Actix-web, minimal implementation)
- **mapadecoletivos-app/** - React frontend (Vite + Three.js + Leaflet/Mapbox)

Each submodule is a separate Git repository. Changes must be committed to the submodule repo first, then the parent repo reference updated.

## Critical Workflows

### Submodule Management (ALWAYS use these)
```bash
# Initial setup or after git clone
./git-commands.sh setup

# Switch all submodules to develop branch
./git-commands.sh update-develop

# Switch all submodules to main/master branch
./git-commands.sh update-main
```

**Never** use standard `npm workspace` or `yarn workspace` commands - this is not a package-based monorepo.

### Development Servers

**API (TypeScript):**
```bash
cd mapadecoletivos-api
yarn install
yarn dev  # ts-node-dev with auto-reload on :3333
```

**Frontend:**
```bash
cd mapadecoletivos-app
yarn install
yarn start  # Vite dev server, NOT create-react-app
```

**Rust API (WIP):**
```bash
cargo run -p mapadecoletivos-api-rust  # Runs from root on :8080
```

### Database Setup (TypeScript API)

Uses **PostgreSQL** (not SQLite despite package.json). Connection configured in `ormconfig.json`:
- Host: localhost:5432
- Database: mapadecoletivos
- User: docker / Pass: mapadecoletivos

**Local development with Docker:**
```bash
cd mapadecoletivos-api
docker-compose up -d  # Starts PostgreSQL + API containers
yarn typeorm migration:run  # Run migrations
```

**Manual setup:**
```bash
cd mapadecoletivos-api
yarn install
yarn typeorm migration:run
yarn dev  # Runs on :3333, debug on :9229
```

## Code Conventions

### API Architecture (mapadecoletivos-api)

**Repository Pattern with TypeORM:**
- Custom repositories in `/repositories/` extend TypeORM's `Repository`
- Use `getCustomRepository(CollectiveRepository)` in controllers
- Models in `/models/` use decorators: `@Entity`, `@Column`, `@OneToMany`

**View Layer Pattern (unusual for Express):**
- `/views/collectives_view.ts` transforms entities before JSON response
- Pattern: `collectiveView.render(entity)` or `.renderMany(entities)`
- Always use views in controllers, never return raw entities

**Route Pattern:**
```typescript
// routes.ts
router.post("/collectives", upload.array('images'), collectiveController.create);
router.get("/collectives/:id", collectiveController.show);
router.get("/collectives", collectiveController.index);
```

**File Uploads:** 
- Multer configured in `/config/upload.ts`
- Images stored in `/uploads/` served statically
- Use `upload.array('images')` middleware for multi-file uploads

**Error Handling:**
- Custom `AppError` class in `/errors/AppError.ts`
- Global error handler middleware in `/errors/Handler.ts`
- `express-async-errors` enables throw in async routes

**Validation:** 
- Yup schemas inline in controllers (not separate files)
- Portuguese error messages: `'Nome é obrigatório'`, `'Sobre é obrigatório'`
- Uses `abortEarly: false` to return all validation errors at once

### Frontend Architecture (mapadecoletivos-app)

**Build Tool:** Vite (ESM-first, fast HMR) - NOT Create React App
- `import.meta.env` for env vars (e.g., `VITE_ACCESS_TOKEN`)
- Top-level await supported

**Styling:** 
- styled-components for component styles
- Global styles in `/styles/global.ts` (not `.css`)
- Page-specific CSS in `/styles/pages/`

**Map Implementation:**
- react-leaflet with Mapbox tiles (requires `VITE_USERNAME`, `VITE_STYLE_ID`, `VITE_ACCESS_TOKEN`)
- Custom map icon in `/utils/mapIcon.ts`

**3D Effects:** 
- Three.js via `@react-three/fiber` and `@react-three/drei`
- Custom components in `/components/Cube/`, `/Discoballs/`
- Mesh gradient renderer for backgrounds

**API Integration:**
```typescript
// services/api.ts
const api = axios.create({ baseURL: "http://localhost:3333" });
```

**Routing:** 
- react-router-dom v6
- Routes configured in `/routes/index.tsx`
- Pages in `/pages/` (Landing, CollectivesMap, CreateCollective, Collective, ErrorPage)

### Rust API (mapadecoletivos-api-rust)

**Status:** Minimal scaffold with hello world endpoint
- Actix-web 4.x
- Runs on 127.0.0.1:8080
- Intended as high-performance alternative to Node API

## Testing & Quality

### Frontend Testing (mapadecoletivos-app)
**Infrastructure configured but no tests written yet:**
- Vitest + @testing-library/react configured in `vite.config.ts`
- jsdom environment for DOM testing
- Coverage reporting enabled (text, json, html)
- Run tests: `yarn test` (watch mode) or `yarn test:coverage`

### API Testing (mapadecoletivos-api)
**No testing infrastructure present** - tests need to be added from scratch

## Docker & Deployment

### Docker Setup (API only)
The Node.js API has full Docker configuration in `mapadecoletivos-api/`:

**docker-compose.yml** orchestrates two services:
- `database` - PostgreSQL 5432 with persistent volume
- `app` - API on port 3333 + debug port 9229

**Usage:**
```bash
cd mapadecoletivos-api
docker-compose up -d     # Start both services
docker-compose down      # Stop all services
docker-compose logs app  # View API logs
```

**Dockerfile details:**
- Base: `node:alpine` (minimal footprint)
- Working directory: `/usr/app`
- Runs `npm run dev` (ts-node-dev with hot reload)
- Volume mounting for live code updates

### Deployment
**No CI/CD or cloud deployment config present** - manual deployment only

## Rust Migration Strategy

The Rust API (`mapadecoletivos-api-rust/`) is currently a **proof-of-concept** with only a hello world endpoint. No feature parity with Node.js API yet. Both APIs are maintained independently - no shared data layer or migration plan documented.

## Key Files

- `git-commands.sh` - Essential submodule management (replaces makefile)
- `Cargo.toml` (root) - Rust workspace configuration
- `mapadecoletivos-api/ormconfig.json` - Database connection & migrations
- `mapadecoletivos-api/docker-compose.yml` - PostgreSQL + API containerization
- `mapadecoletivos-api/Dockerfile` - API container definition
- `mapadecoletivos-app/vite.config.ts` - Build & test configuration
- `mapadecoletivos-api/src/app.ts` - Express app setup with CORS middleware

## Gotchas

1. **Submodule commits:** Always commit inside submodule first, then update parent repo
2. **Database mismatch:** Code uses PostgreSQL despite sqlite3 in package.json dependencies
3. **Commented code:** Frontend has commented-out API fetching (see CollectivesMap.tsx:36-42)
4. **Branch strategy:** API and App use `develop` branch; Rust API uses `main`
5. **Environment vars:** Frontend requires Mapbox credentials in `.env.local` (VITE_* prefix)
