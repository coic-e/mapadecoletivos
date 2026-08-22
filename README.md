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
| Postgres 16 | `rave-map-db` | 5432 | usuário `docker`, banco `rave_map` |
| RustFS | `rave-map-rustfs` | 9000 / 9001 | bucket das imagens; 9001 é o console web |
| API Rust | `rave-map-rust-api` | 8080 | espera banco e bucket ficarem prontos |

As portas ficam publicadas só em `127.0.0.1`, e o compose exige um `.env` na raiz — copie de `.env.example` e preencha. Não há valor padrão para segredo: a API recusa subir com os de exemplo.

As imagens dos cadastros vão para um bucket compatível com S3, não para o disco do container. O RustFS cobre tanto o ambiente local quanto a produção; trocar por um serviço gerenciado (Cloudflare R2, S3, Spaces) é questão de mudar endpoint e credenciais, sem tocar em código.

O MinIO saiu: o console foi removido da edição comunitária em 2025 e o repositório foi arquivado em abril de 2026.

**A política do bucket precisa liberar só `s3:GetObject`.** Liberar `ListBucket` junto entrega o nome de toda imagem já enviada, inclusive de cadastro pendente ou rejeitado. O `rustfs-init` do compose já aplica a política certa.

### Produção com RustFS

O bucket precisa ser **alcançável pelo navegador de quem visita o site**, porque é dele que as fotos são baixadas. Na prática:

- um domínio com HTTPS apontando para a porta 9000 do RustFS (o proxy do EasyPanel resolve o certificado). Página em https com imagem em http é bloqueada como conteúdo misto;
- esse domínio vai em `S3_PUBLIC_BASE_URL` na API **e** em `VITE_IMAGES_BASE_URL` no front, que o coloca no `img-src` do CSP;
- o console (9001) não deve ficar exposto na internet;
- o volume de dados do RustFS passa a guardar as fotos dos cadastros: entra no backup junto com o Postgres.

### Alternativa: Cloudflare R2

| Variável | Valor |
|---|---|
| `S3_ENDPOINT` | `https://<ACCOUNT_ID>.r2.cloudflarestorage.com` |
| `S3_REGION` | `auto` (`us-east-1` e vazio são apelidos dele) |
| `S3_PUBLIC_BASE_URL` | o domínio público do bucket, **não** o endpoint da API |

O R2 **não tem ACL nem bucket policy**: acesso público se liga pelo subdomínio `r2.dev` do bucket ou, de preferência, por um domínio próprio. Isso elimina a armadilha do `ListBucket` — esses endereços servem objeto por objeto e não listam nada.

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
