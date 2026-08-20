# Mapa de Rave

Monorepo do Mapa de Rave: um mapa aberto da cena de música eletrônica brasileira — coletivos, festas, labels, clubs, rádios e produtoras.

## Estrutura

```
mapadecoletivos/
├── app/              # Front-end React + Vite (o site)
├── api-rust/         # API em Rust (Actix-web + Diesel) — a API atual
├── api-types/        # Crate: tipos de resposta da API (views serializadas)
├── db-types/         # Crate: modelos do banco (Diesel)
├── api-legada/       # API antiga em Node/TypeORM — mantida só como referência
├── uploads/          # Imagens enviadas pelo cadastro (bind mount do container)
├── Cargo.toml        # Workspace Rust (api-rust, api-types, db-types)
├── Dockerfile        # Build da API Rust; o contexto é a raiz do workspace
└── docker-compose.yml # Postgres + API Rust
```

> `api-legada/` está congelada e será removida. Nada novo deve ser escrito lá.

## Subindo o ambiente

### Banco e API, via Docker

```bash
docker compose up -d
```

Sobe dois serviços:

| Serviço | Container | Porta | Detalhes |
|---|---|---|---|
| Postgres 16 | `rave-map-db` | 5432 | usuário `docker`, senha `ravemap`, banco `rave_map` |
| API Rust | `rave-map-rust-api` | 8080 | espera o healthcheck do banco |

A imagem da API é Debian slim, e não Alpine, porque o Diesel linka com `libpq`.

> A API não sobe sem `JWT_SECRET`, que assina os tokens dos moderadores. O compose tem um valor de desenvolvimento; em produção passe pelo ambiente.

### Front-end

```bash
cd app
cp .env.example .env    # preencha as credenciais do Mapbox
npm install
npm start               # http://localhost:5173
```

O app espera a API em `http://localhost:8080`. Detalhes de variáveis, scripts e stack em [`app/README.md`](app/README.md).

### API fora do Docker

```bash
cd api-rust
cp .env.example .env
diesel migration run
cargo run
```

Documentação completa da API — endpoints, payloads, estrutura — em [`api-rust/README.md`](api-rust/README.md).

## Workspace Rust

Os comandos rodam da raiz e valem para os três crates:

```bash
cargo build            # compila o workspace
cargo test             # roda os testes
cargo check            # checa sem compilar binário
cargo run -p api-rust  # sobe só a API
cargo fmt && cargo clippy
```

`api-types` e `db-types` existem para separar o que é modelo de banco do que é resposta HTTP: `db-types` guarda as structs do Diesel, `api-types` guarda as views que a API serializa. O front-end só conhece as segundas.

## Fluxo de trabalho

O repositório foi consolidado: `app`, `api-rust` e `api-legada` vivem aqui como diretórios normais. **Não há mais submódulos** — se você seguiu um README antigo falando em `git submodule` ou `git-commands.sh`, esqueça, foi tudo removido.

```bash
git checkout -b feat/nome-da-mudanca
# ... mude o que precisa ...
git commit
gh pr create
```

Antes de abrir PR, no que você mexeu:

```bash
cd app && npm run lint && npx tsc --noEmit && npx vitest run && npm run build
cargo clippy && cargo test
```

## Licença

MIT — mas o arquivo `LICENSE` ainda não existe no repositório. Vale adicionar antes de divulgar o projeto como open source.
