# Mapadecoletivos AI Coding Guide

## Project Architecture

This is a **Git submodules monorepo** (not Lerna/Turborepo) containing three independent services for mapping electronic music collectives/organizations in Brazil:

- **api/** - TypeScript/Node.js REST API (Express + TypeORM + PostgreSQL)
- **api-rust/** - Rust API implementation (Actix-web + Diesel ORM + PostgreSQL)
- **app/** - React frontend (Vite + Leaflet/Mapbox)

Each submodule is a separate Git repository. Changes must be committed to the submodule repo first, then the parent repo reference updated.

**Terminology Note:** The project is transitioning from "collectives" to "organizations" terminology:
- Node API still uses `/collectives` routes and `Collective` models
- Rust API uses `/organizations` routes and `Organization` models  
- Frontend uses "organizations" in code but routes under `/raves` path

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

**Never** use standard `npm workspace` commands - this is not a package-based monorepo.

### Development Servers

**API (TypeScript):**
```bash
cd api
npm install
npm run dev  # ts-node-dev with auto-reload on :3333
```

**Frontend:**
```bash
cd app
npm install
npm start  # Vite dev server (NOT create-react-app)
```

**Rust API:**
```bash
cd api-rust
cargo run  # Runs on 0.0.0.0:8080
# Or from root: cargo run -p api-rust
```

### Database Setup

**Both APIs share the same PostgreSQL database**:
- Host: localhost:5432
- Database: mapadecoletivos
- User: docker / Pass: mapadecoletivos

**TypeScript API (ormconfig.json):**
```bash
cd api
docker-compose up -d  # Starts PostgreSQL + API containers
npm run typeorm migration:run  # Run migrations
npm run dev  # Runs on :3333, debug on :9229
```

**Rust API (diesel.toml):**
```bash
cd api-rust
# Install diesel CLI if needed: cargo install diesel_cli --no-default-features --features postgres
docker-compose up -d  # Starts PostgreSQL
diesel migration run  # Run migrations
cargo run  # Runs on :8080
```

## Code Conventions

### Node.js API Architecture (api/)

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

### Frontend Architecture (app/)

**Build Tool:** Vite (ESM-first, fast HMR) - NOT Create React App
- `import.meta.env` for env vars (e.g., `VITE_ACCESS_TOKEN`)
- Top-level await supported
- Environment config centralized in `src/config/env.ts`

**Styling:** 
- CSS modules (not styled-components)
- Global styles in `/styles/global.css`
- Page-specific CSS in `/styles/pages/`

**Map Implementation:**
- react-leaflet with Mapbox tiles (requires `VITE_USERNAME`, `VITE_STYLE_ID`, `VITE_ACCESS_TOKEN`)
- Custom map icon in `/utils/mapIcon.ts`
- Center coordinates: Brazil-centered at [-13.702797, -50.6865109]

**API Integration:**
```typescript
// services/api.ts
const api = axios.create({ baseURL: "http://localhost:3333" });
```

**Routing:** 
- react-router-dom v7 (createBrowserRouter API)
- Routes configured in `/routes/index.tsx`
- Pages in `/pages/`: Landing, OrganizationsMap, CreateOrganization, Organization, ErrorPage
- URL structure: `/` (landing), `/raves` (map), `/raves/create`, `/raves/:id`

**Validation:**
- Zod for schema validation (replacing Yup gradually)

### Rust API Architecture (api-rust/)

**Status:** Fully implemented with feature parity to Node API
- Actix-web 4.x web framework
- Diesel 2.x ORM with PostgreSQL
- Runs on 0.0.0.0:8080

**Domain-Driven Structure:**
```
src/
├── domains/organizations/  # Domain logic
│   ├── actions.rs         # Business logic
│   ├── repository.rs      # Database operations
│   └── routes.rs          # HTTP handlers
├── models/                # Database entities
├── views/                 # JSON serialization
├── handlers/              # Shared handlers (upload)
└── errors/                # Error types
```

**Key Features:**
- REST endpoints: GET/POST `/organizations`, GET `/organizations/:id`
- Multipart file upload with validation (max 10MB)
- Portuguese validation messages matching Node API
- Pagination support (limit/offset query params)
- Static file serving for `/uploads`
- Configuration via environment variables
- Health check endpoint at `/health`

**Error Handling:**
- Custom `ApiError` enum with thiserror
- Automatic HTTP status code mapping
- Structured validation errors

**Database Migrations:**
- Diesel migrations in `/migrations/`
- `2026-02-28-212003_create_organizations`
- `2026-02-28-212010_create_images`

## Testing & Quality

### Frontend Testing (app/)
**Infrastructure configured but no tests written yet:**
- Vitest + @testing-library/react configured in `vite.config.ts`
- jsdom environment for DOM testing
- Coverage reporting enabled (text, json, html)
- Run tests: `npm test` (watch mode) or `npm test:coverage`

### API Testing
- **Node API (api/):** No testing infrastructure - tests need to be added from scratch
- **Rust API (api-rust/):** Basic integration test scaffold in `/tests/integration_test.rs`

## Docker & Deployment

### Docker Setup

**Node.js API (`api/docker-compose.yml`):**
- `database` service - PostgreSQL 5432 with persistent volume
- `app` service - API on port 3333 + debug port 9229
- Base image: `node:alpine`
- Working directory: `/usr/app`
- Hot reload via volume mounting

**Rust API (`api-rust/docker-compose.yml`):**
- `postgres` service - PostgreSQL 5432
- `api` service - Rust API on port 8080
- Base image: `rust:1.75-slim`
- Multi-stage build for smaller production image

**Usage:**
```bash
cd api  # or api-rust
docker-compose up -d     # Start services
docker-compose down      # Stop services
docker-compose logs app  # View API logs (or 'api' for Rust)
```

### Deployment
**No CI/CD or cloud deployment config present** - manual deployment only

## API Migration Strategy

Both APIs are **fully functional and maintained in parallel**:
- **Shared database**: Both connect to same PostgreSQL instance
- **Different schemas**: Node uses TypeORM migrations, Rust uses Diesel migrations  
- **Terminology divergence**: Node uses "collectives", Rust uses "organizations"
- **Port allocation**: Node (:3333), Rust (:8080)
- **No planned deprecation**: Both are actively developed

## Key Files

- `git-commands.sh` - Essential submodule management (preferred over makefile)
- `makefile` - Legacy submodule commands (use git-commands.sh instead)
- `Cargo.toml` (root) - Rust workspace configuration (members: ["api-rust"])
- `api/ormconfig.json` - TypeORM database connection & migrations
- `api/docker-compose.yml` - PostgreSQL + Node API containerization
- `api/src/app.ts` - Express app setup with CORS middleware
- `api-rust/diesel.toml` - Diesel configuration
- `api-rust/src/main.rs` - Actix-web server initialization
- `app/vite.config.ts` - Build & test configuration
- `app/src/config/env.ts` - Environment variable validation

## Gotchas

1. **Submodule commits:** Always commit inside submodule first, then update parent repo
2. **Terminology inconsistency:** "collectives" (Node API) vs "organizations" (Rust API, Frontend code) vs "/raves" (Frontend URLs)
3. **Database dependency:** Code uses PostgreSQL despite sqlite3 in package.json (legacy dependency)
4. **Commented code:** Frontend uses hardcoded mock data instead of API calls (see OrganizationsMap.tsx)
5. **Branch strategy:** API and App use `develop` branch; Rust API uses `main`; Parent repo uses `main`
6. **Environment vars:** 
   - Frontend requires Mapbox credentials (VITE_USERNAME, VITE_STYLE_ID, VITE_ACCESS_TOKEN)
   - Rust API needs DATABASE_URL, SERVER_HOST, SERVER_PORT, UPLOAD_DIR, BASE_URL
7. **Folder naming:** Folders are `api/`, `app/`, `api-rust/` (not prefixed with `mapadecoletivos-`)
8. **Shared database:** Both APIs can write to same DB - be careful with schema changes
