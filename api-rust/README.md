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
../db-types/                   # Crate: database types
└── src/
    ├── schema.rs             # Diesel schema (auto-generated)
    ├── organization.rs       # Organization entity & validation
    └── image.rs              # Image entity
../api-types/                  # Crate: API request/response types
└── src/
    ├── organization_view.rs  # JSON transformation for organizations
    └── image_view.rs         # JSON transformation for images
src/
├── main.rs                    # Application entry point
├── lib.rs                     # Library exports for testing
├── config.rs                  # Configuration from environment
├── db.rs                      # Database connection pool
├── errors/
│   └── api_error.rs          # Custom error types
├── domains/
│   └── organizations/
│       ├── routes.rs         # HTTP request handlers
│       ├── actions.rs        # Business logic
│       └── repository.rs     # Database operations
└── handlers/
    └── upload.rs             # File upload processing
migrations/
├── *_create_organizations/   # Database migration for organizations table
└── *_create_images/          # Database migration for images table
```

## Prerequisites

- Rust 1.78+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 12+ (running locally or via Docker)
- Diesel CLI: `cargo install diesel_cli --no-default-features --features postgres`

## Setup

### 1. Clone and Navigate

```bash
cd api-rust
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
DATABASE_URL=postgresql://docker:ravemap@localhost:5432/rave_map
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
UPLOAD_DIR=uploads
MAX_FILE_SIZE=10485760  # 10MB
BASE_URL=http://localhost:8080
RUST_LOG=info
```

### 3. Set Up Database

Start PostgreSQL with Docker and run migrations:

```bash
docker-compose up -d database  # Starts PostgreSQL
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
./target/release/api-rust
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
    "latitude": -23.55,
    "longitude": -46.63,
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

`GET /organizations/{id_ou_slug}` aceita as duas formas: numérico é id, o resto é slug. Os dois existem porque o site passou a linkar por slug, mas links antigos com id continuam circulando por aí.

```http
GET /collectives/:id
```

Returns single collective with images.

**Response:** Same structure as array item above, or 404 if not found.

### Create Collective

```http
POST /organizations
Content-Type: multipart/form-data
```

Cria com `status = "pending"`: o cadastro só aparece no site depois que um moderador aprova.

**Obrigatórios**

| Campo | Regra |
|---|---|
| `name` | 1 a 120 caracteres |
| `latitude` / `longitude` | numéricos, dentro da faixa do globo |
| `type` | um de: Festa, Festival, Label, Radio, Podcast, Coletivo, Nucleo, Club, Bar, Produtora, outro |
| `city` | 1 a 120 caracteres |
| `uf` | 2 letras |
| `email` | e-mail válido |
| `about` | 10 a 1200 caracteres |
| `genres` | um ou mais da lista fechada, **separados por vírgula num campo só** — os campos do multipart viram um mapa, então nome repetido se sobrescreveria |
| `images` | ao menos um arquivo |

**Opcionais:** `address`, `instagram`, `soundcloud`, `bandcamp`, `youtube`, `spotify`, `website`, `frequency` (Semanal, Quinzenal, Mensal, Sazonal, Pontual), `is_active` (ausente significa ativo) e `cover_index` (qual das fotos enviadas é a capa, pela ordem de envio; ausente ou fora da faixa usa a primeira).

O `slug` é derivado do nome na criação e devolvido na resposta. Nome repetido ganha sufixo numérico — `deposito-42`, `deposito-42-2` — resolvido dentro da transação para duas requisições simultâneas não saírem com o mesmo slug.

**Regra que cruza campos:** pelo menos um dos seis links precisa vir preenchido, senão o cadastro não serve para achar o rolê. O erro volta em `errors.__all__`, não preso a um campo.

Os gêneros aceitos estão em `MUSIC_GENRES`, em `db-types/src/organization.rs`. A lista é fechada dos dois lados porque é ela que sustenta o filtro do mapa: texto livre viraria "techno", "Techno" e "tekno" como três coisas diferentes.

### Healthcheck

São duas sondas, porque respondem perguntas diferentes.

| Rota | Responde | Verifica |
|---|---|---|
| `GET /health` | 200 sempre que o processo está de pé | nada externo |
| `GET /health/ready` | 200 ou **503** | banco e bucket, com teto de 3s cada |

```json
// GET /health/ready com o bucket fora do ar
{ "status": "unavailable", "database": "ok", "storage": { "error": "tempo esgotado" } }
```

Use `/health` para política de **reinício**: derrubar o container porque o Postgres piscou não conserta nada e ainda tira a API do ar junto. Use `/health/ready` para decidir **roteamento de tráfego** — é o que diz se esta instância consegue atender.

No EasyPanel, o campo de health check é o `/health/ready` se ele apenas marcar o serviço como indisponível; se ele reiniciar o container ao falhar, aponte para `/health`.

A resposta diz **qual** dependência falhou, de propósito: sonda que só responde "não" obriga quem está de plantão a adivinhar.

## Imagens

Ficam num bucket compatível com S3, nunca no disco do container — disco de container é efêmero e não é compartilhado entre réplicas.

O upload é validado por *magic bytes* antes de subir: o formato sai do conteúdo, não do nome nem do content-type que o cliente mandou, e SVG é recusado por ser documento que executa script. O nome do objeto é sorteado, o content-type gravado vem da detecção, e envio que falha no meio tem os objetos já subidos apagados.

O banco guarda **só a chave do objeto**; a URL é montada na resposta a partir de `S3_PUBLIC_BASE_URL`. Trocar de provedor ou pôr um CDN na frente não exige migração de dados.

| Variável | Para quê |
|---|---|
| `S3_ENDPOINT` | vazio para AWS S3; `http://rustfs:9000` no compose; `https://<ACCOUNT_ID>.r2.cloudflarestorage.com` no R2 |
| `S3_BUCKET` | nome do bucket |
| `S3_ACCESS_KEY` / `S3_SECRET_KEY` | credenciais |
| `S3_PUBLIC_BASE_URL` | como o navegador chega nas imagens (CDN, se houver) |
| `S3_REGION` | padrão `us-east-1` |
| `S3_FORCE_PATH_STYLE` | padrão: ligado quando há endpoint, desligado na AWS |

Em produção, a política do bucket deve permitir apenas `s3:GetObject` em `arn:aws:s3:::<bucket>/*`. Liberar `ListBucket` expõe o nome de toda imagem enviada, inclusive de cadastros que a moderação nunca aprovou.

## Moderação

Cadastro criado pelo site nasce com `status = "pending"` e **não aparece nas rotas públicas**. `GET /organizations` e `GET /organizations/{id}` só enxergam `approved` — um cadastro pendente responde 404, e não "existe mas está escondido".

Os três estados são `pending`, `approved` e `rejected`, garantidos por um CHECK no banco.

### Criando o primeiro moderador

Não há rota pública para criar admin. Há dois caminhos.

**Pelo ambiente**, que é o que serve para deploy sem console:

```bash
ADMIN_NAME=Moderação
ADMIN_EMAIL=voce@dominio.com
ADMIN_PASSWORD=            # mínimo 12 caracteres
```

A API cria a conta na subida **apenas quando não existe nenhum moderador**. É semente de primeira subida, não fonte permanente de senha: se aplicasse a senha a cada boot, quem tivesse acesso ao painel assumiria a conta editando a variável e reiniciando o serviço. Nas subidas seguintes ela é ignorada e o log avisa para removê-la do ambiente — e vale remover mesmo, porque variável de ambiente aparece em `docker inspect` e em `/proc/<pid>/environ`.

Senha menor que 12 caracteres **derruba a subida**, em vez de criar uma conta fraca em silêncio.

**Pelo binário**, para criar moderadores depois do primeiro:

```bash
create_admin "Nome" email@dominio.com
```

A senha vem pela entrada padrão, nunca por argumento — argumento de processo aparece na lista de processos e no histórico do shell. Funciona em pipe também: `echo "senha" | create_admin ...`.

### Autenticação

```http
POST /auth/login
{ "email": "...", "password": "..." }
→ 200 { "token": "<jwt>", "admin": { "id": 1, "name": "...", "email": "..." } }
→ 401 { "message": "E-mail ou senha inválidos" }
```

O token vai nas rotas administrativas como `Authorization: Bearer <jwt>` e vale `JWT_TTL_HOURS` horas (12 por padrão). `GET /auth/me` devolve o admin do token — o painel usa para saber se a sessão ainda vale.

Senha é guardada com argon2. E-mail inexistente e senha errada devolvem a mesma mensagem, de propósito: a diferença revelaria quais e-mails existem.

### Rotas de moderação

Todas exigem Bearer token; sem ele, 401.

| Rota | O que faz |
|---|---|
| `GET /admin/organizations?status=pending` | Fila de revisão. `status` aceita `pending`, `approved`, `rejected` ou `all`; o padrão é `pending`. Mais antigos primeiro. |
| `GET /admin/organizations/{id}` | Detalhe, em qualquer estado |
| `POST /admin/organizations/{id}/approve` | Passa a aparecer no site |
| `POST /admin/organizations/{id}/reject` | Sai do site. Corpo opcional: `{ "reason": "..." }` |

Aprovação e rejeição gravam `reviewed_at` e `reviewed_by`, então dá para saber quem decidiu o quê e quando.

## Development

### Run Tests

```bash
# Unit tests (no database required)
cargo test

# Integration tests (requires database)
DATABASE_URL=postgresql://docker:ravemap@localhost:5432/rave_map cargo test --features integration_tests
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
- Start PostgreSQL database
- Expose API on port 8080
- Mount `./uploads` directory

### Manual Docker Build

```bash
docker build -t mapadecoletivos-rust-api .
docker run -p 8080:8080 --env-file .env mapadecoletivos-rust-api
```

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

## Troubleshooting

### Database connection failed
```
Error: Connection refused port 5432
```
**Solution**: Ensure PostgreSQL is running:
```bash
docker-compose up -d database
```

### Migrations already ran
```
Error: Migration X has already been run
```
**Solution**: Skip if database already has tables.

### Port already in use
```
Error: Address already in use (os error 98)
```
**Solution**: Change `SERVER_PORT` in `.env` or stop conflicting service.

## Contributing

This is part of the Mapadecoletivos monorepo. See main README for contribution guidelines.

## License

See root LICENSE file.
