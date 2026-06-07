# Mapadecoletivos API - Rust

High-performance Rust implementation of the Mapadecoletivos API using Actix-web and Diesel ORM.

## Features

- ✅ REST API with 3 endpoints (GET /collectives, GET /collectives/:id, POST /collectives)
- ✅ PostgreSQL database with Diesel ORM
- ✅ File upload support with validation
- ✅ Custom view/serialization layer
- ✅ Portuguese validation messages
- ✅ Error handling with proper HTTP status codes
- ✅ CORS support
- ✅ Static file serving for uploaded images
- ✅ Pagination support for list endpoint
- ✅ Configurable via environment variables
- ✅ Docker support

## Architecture

### Tech Stack

- **Web Framework**: Actix-web 4.x
- **ORM**: Diesel 2.x with PostgreSQL
- **Validation**: Validator crate with custom Portuguese messages
- **Serialization**: Serde with custom view layer
- **File Upload**: Actix-multipart with disk storage
- **Error Handling**: Thiserror with custom ApiError enum

### Project Structure

```
src/
├── main.rs                    # Application entry point
├── lib.rs                     # Library exports for testing
├── config.rs                  # Configuration from environment
├── db.rs                      # Database connection pool
├── schema.rs                  # Diesel schema (auto-generated)
├── errors/
│   └── api_error.rs          # Custom error types
├── models/
│   ├── collective.rs         # Collective entity & validation
│   └── image.rs              # Image entity
├── repositories/
│   ├── collective_repository.rs  # Database operations for collectives
│   └── image_repository.rs       # Database operations for images
├── views/
│   ├── collective_view.rs    # JSON transformation for collectives
│   └── image_view.rs         # JSON transformation for images
└── handlers/
    ├── collective_handler.rs  # HTTP request handlers
    └── upload.rs             # File upload processing
migrations/
├── *_create_collectives/     # Database migration for collectives table
└── *_create_images/          # Database migration for images table
```

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 12+ (running locally or via Docker)
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

## Setup

### 1. Clone and Navigate

```bash
cd mapadecoletivos-api-rust
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
DATABASE_URL=postgresql://docker:mapadecoletivos@localhost:5432/mapadecoletivos
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
UPLOAD_DIR=uploads
MAX_FILE_SIZE=10485760  # 10MB
BASE_URL=http://localhost:8080
RUST_LOG=info
```

### 3. Set Up Database

**Option A: Use existing database** from TypeScript API (shared):

```bash
# The database should already exist from mapadecoletivos-api
# Just run migrations
diesel migration run
```

**Option B: Start fresh database** with Docker:

```bash
cd ../mapadecoletivos-api
docker-compose up -d database  # Starts PostgreSQL
cd ../mapadecoletivos-api-rust
diesel setup                   # Creates database and runs migrations
```

### 4. Run Application

```bash
cargo run
```

Server starts on `http://0.0.0.0:8080`

For production build:

```bash
cargo build --release
./target/release/mapadecoletivos-api-rust
```

## API Endpoints

### List Collectives

```http
GET /collectives?limit=10&offset=0
```

Returns array of collectives with images. Supports pagination.

**Response:**
```json
[
  {
    "id": 1,
    "name": "Coletivo Example",
    "latitude": "-23.55",
    "longitude": "-46.63",
    "type": "Sound System",
    "city": "São Paulo",
    "uf": "SP",
    "email": "contact@example.com",
    "social": "https://instagram.com/example",
    "about": "Description...",
    "images": [
      {
        "id": 1,
        "url": "http://localhost:8080/uploads/1234567890-image.jpg"
      }
    ]
  }
]
```

### Get Single Collective

```http
GET /collectives/:id
```

Returns single collective with images.

**Response:** Same structure as array item above, or 404 if not found.

### Create Collective (Partially Implemented)

```http
POST /collectives
Content-Type: multipart/form-data
```

**Note:** This endpoint requires additional work to fully parse multipart form data with text fields. Currently handles file uploads but text field extraction needs implementation.

**Expected fields:**
- `name` (required)
- `latitude` (required, numeric)
- `longitude` (required, numeric)
- `type` (required)
- `city` (required)
- `uf` (required)
- `email` (required, valid email)
- `social` (required)
- `about` (required, max 300 chars)
- `images[]` (file uploads)

### Healthcheck

```http
GET /health
```

Returns `OK` status.

## Development

### Run Tests

```bash
# Unit tests (no database required)
cargo test

# Integration tests (requires database)
DATABASE_URL=postgresql://docker:mapadecoletivos@localhost:5432/mapadecoletivos cargo test --features integration_tests
```

### Check Code

```bash
cargo check           # Fast compilation check
cargo clippy          # Linter suggestions
cargo fmt             # Format code
```

### Database Migrations

```bash
# Create new migration
diesel migration generate migration_name

# Run pending migrations
diesel migration run

# Rollback last migration
diesel migration revert

# Regenerate schema.rs
diesel migration run
```

## Docker Deployment

### Build and Run with Docker Compose

```bash
docker-compose up -d
```

This will:
- Build Rust API container
- Connect to shared PostgreSQL database
- Expose API on port 8080
- Mount `./uploads` directory

### Manual Docker Build

```bash
docker build -t mapadecoletivos-rust-api .
docker run -p 8080:8080 --env-file .env mapadecoletivos-rust-api
```

## Coexistence with TypeScript API

Both APIs can run simultaneously:

- **TypeScript API**: Port 3333 (http://localhost:3333)
- **Rust API**: Port 8080 (http://localhost:8080)

Both share the same PostgreSQL database (`mapadecoletivos`). Frontend can be configured to use either by changing the base URL.

## Implementation Status

### ✅ Completed
- Database models and migrations
- Repository layer with CRUD operations
- View/serialization layer with proper JSON transformation
- Error handling system with Portuguese validation messages
- GET endpoints (index, show) with pagination
- Static file serving for uploads
- CORS middleware
- Logging middleware
- Environment-based configuration
- Docker configuration
- Integration test structure

### ⚠️ Needs Work
- **POST /collectives**: Multipart form data parsing needs completion
  - File upload logic is implemented
  - Need to extract and parse text fields from multipart payload
  - Need to integrate validation before file saving
  
- **Cleanup logic**: Implement orphaned file cleanup on failed transactions

### 🔜 Future Enhancements
- Authentication/Authorization
- Rate limiting
- Request validation middleware
- Update/Delete endpoints
- Image compression/optimization
- Database connection pooling optimization
- Comprehensive integration test suite
- Performance benchmarks vs TypeScript API

## Differences from TypeScript API

### Improvements
1. **Type Safety**: Compile-time type checking prevents runtime errors
2. **Performance**: Significantly faster request handling and lower memory usage
3. **Configurable URLs**: Base URL is environment-configurable (fixes hardcoded localhost:3333)
4. **Validation-first uploads**: Files can be saved after validation (vs TypeScript saves first)
5. **Pagination**: Built-in pagination support for list endpoint
6. **Better error handling**: Structured error responses with proper HTTP status codes

### Maintained Compatibility
- Same database schema
- Same JSON response structure
- Same validation messages (Portuguese)
- Same file naming convention
- Same endpoint paths

## Troubleshooting

### Database connection failed
```
Error: Connection refused port 5432
```
**Solution**: Ensure PostgreSQL is running:
```bash
cd ../mapadecoletivos-api
docker-compose up -d database
```

### Migrations already ran
```
Error: Migration X has already been run
```
**Solution**: Migrations are shared with TypeScript API. Skip if database already has tables.

### Port already in use
```
Error: Address already in use (os error 98)
```
**Solution**: Change `SERVER_PORT` in `.env` or stop conflicting service.

## Contributing

This is part of the Mapadecoletivos monorepo. See main README for contribution guidelines.

## License

See root LICENSE file.
