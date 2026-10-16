# API do Mapa de Rave

API em Rust — Actix-web 4 e Diesel 2 sobre Postgres. Serve o mapa público, recebe os cadastros e sustenta a moderação.

## Estrutura

```
../db-types/          # Modelos do Diesel e o schema. Nada aqui é serializável para HTTP.
../api-types/         # Views: o que a API serializa. É só isto que o front conhece.
src/
├── main.rs           # Sobe migrações, pool e servidor, nessa ordem
├── app.rs            # Monta o App do actix; compartilhado com os testes
├── config.rs         # Configuração vinda do ambiente, validada na subida
├── db.rs             # Pool r2d2
├── migrations.rs     # Migrações embutidas no binário
├── bootstrap.rs      # Semente do primeiro moderador
├── storage.rs        # Bucket S3
├── rate_limit.rs     # Limite por IP, em memória
├── auth/             # Senha, JWT e o extractor AdminIdentity
├── errors/           # ApiError e a tradução para status HTTP
├── handlers/upload.rs
└── domains/
    ├── organizations/   routes · actions · repository · auth
    ├── edit_requests/   routes · actions · repository · auth
    └── admins/          routes · actions · repository
```

Cada domínio tem quatro arquivos com papéis fixos: `routes` fala HTTP, `actions` tem a regra e a transação, `repository` fala com o banco, `auth` diz quem pode o quê. `db-types` e `api-types` existem para que linha de banco e resposta de API não sejam o mesmo tipo — um campo novo no banco não vaza para o JSON sozinho.

## Rodando

Pelo compose, da raiz do repositório:

```bash
docker compose up -d
```

Fora do Docker:

```bash
cp .env.example .env
cargo run -p api-rust
```

As migrações rodam **na subida da API**, não no build: não há passo manual depois do deploy. Falha ao migrar derruba o processo em vez de servir com o schema errado.

## Variáveis de ambiente

Sem as obrigatórias a API não sobe, e o erro lista todas as que faltam de uma vez.

| Variável | Padrão | Para quê |
|---|---|---|
| `DATABASE_URL` | — **obrigatória** | Postgres |
| `JWT_SECRET` | — **obrigatória** | assina os tokens; mínimo 32 caracteres, e os valores de exemplo do repositório são recusados |
| `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY` | — **obrigatórias** | bucket das imagens |
| `S3_ENDPOINT` | vazio | vazio na AWS; `http://rustfs:9000` no compose; `https://<ID>.r2.cloudflarestorage.com` no R2 |
| `S3_PUBLIC_BASE_URL` | — **obrigatória** | como o **navegador** chega nas imagens |
| `S3_REGION` | `us-east-1` | no R2, `auto` |
| `S3_FORCE_PATH_STYLE` | ligado quando há endpoint | AWS usa subdomínio; RustFS e MinIO usam caminho |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:5173` | lista separada por vírgula. `*` é recusado |
| `SERVER_HOST` / `SERVER_PORT` | `0.0.0.0` / `8080` | o processo roda sem root, então não use porta abaixo de 1024 |
| `JWT_TTL_HOURS` | `8` | validade da sessão do moderador |
| `MAX_FILE_SIZE` | 5 MB | por arquivo |
| `MAX_FILES_PER_REQUEST` | `6` | fotos por cadastro |
| `MAX_REQUEST_SIZE` | 32 MB | corpo inteiro |
| `MAX_FIELD_SIZE` | 16 KB | por campo de texto |
| `LOGIN_RATE_LIMIT` | `5` | tentativas de login por janela, por IP |
| `SUBMISSION_RATE_LIMIT` | `10` | cadastros por janela, por IP |
| `RATE_LIMIT_WINDOW_SECS` | `300` | tamanho da janela |
| `TRUST_PROXY` | desligado | ligue **só** atrás de proxy que escreve `X-Forwarded-For`; ligado sem proxy, qualquer um forja o próprio IP e escapa do limite |
| `ADMIN_NAME`, `ADMIN_EMAIL`, `ADMIN_PASSWORD` | — | semente do primeiro moderador; ver abaixo |

O limite de taxa vive **em memória do processo**: com mais de uma réplica cada uma conta o seu, e reiniciar zera. Para o volume de hoje serve; se crescer, o estado precisa sair para o Redis.

> `BASE_URL` ainda é lida, mas só aparece numa linha de log na subida. Nada depende dela.

## Rotas

### Públicas

| Rota | O que faz |
|---|---|
| `GET /organizations` | O mapa. Só `approved`. Aceita `limit` e `offset` |
| `GET /organizations/{id_ou_slug}` | Numérico é id, o resto é slug — os dois porque o site linka por slug e links antigos com id ainda circulam |
| `POST /organizations` | Cadastro novo, nasce pendente |
| `POST /organizations/{id_ou_slug}/edit-requests` | Sugestão de correção, sem login |
| `GET /health` · `GET /health/ready` | Sondas |

### De moderação

Todas exigem `Authorization: Bearer <jwt>`; sem ele, 401.

| Rota | O que faz |
|---|---|
| `POST /auth/login` | `{ "email", "password" }` → `{ "token", "admin" }` |
| `GET /auth/me` | O admin do token; o painel usa para saber se a sessão vale |
| `GET /admin/organizations?status=` | Fila. `pending` (padrão), `approved`, `rejected` ou `all`. Mais antigos primeiro |
| `GET /admin/organizations/{id}` | Detalhe, em qualquer estado |
| `PATCH /admin/organizations/{id}` | Edição direta. Campo ausente fica como está, e o cadastro não volta para a fila |
| `POST /admin/organizations/{id}/approve` | Passa a aparecer no site |
| `POST /admin/organizations/{id}/reject` | Sai do site **e apaga as imagens do bucket**. Corpo opcional: `{ "reason": "..." }` |
| `GET /admin/edit-requests?status=` | Fila de sugestões |
| `POST /admin/edit-requests/{id}/apply` | Aplica a sugestão ao cadastro |
| `POST /admin/edit-requests/{id}/reject` | Descarta |

## Cadastro

```http
POST /organizations
Content-Type: multipart/form-data
```

Nasce com `status = "pending"`: só aparece no site depois que um moderador aprova.

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

**Opcionais:** `address`, `instagram`, `soundcloud`, `bandcamp`, `youtube`, `spotify`, `website`, `frequency` (Semanal, Quinzenal, Mensal, Sazonal, Pontual), `is_active` (ausente significa ativo) e `cover_index` (qual das fotos é a capa, pela ordem de envio; ausente ou fora da faixa usa a primeira).

O `slug` é derivado do nome na criação e devolvido na resposta. Nome repetido ganha sufixo numérico — `deposito-42`, `deposito-42-2` — resolvido dentro da transação para duas requisições simultâneas não saírem com o mesmo slug.

**Regra que cruza campos:** pelo menos um dos seis links precisa vir preenchido, senão o cadastro não serve para achar o rolê. O erro volta em `errors.__all__`, não preso a um campo.

Os gêneros aceitos estão em `MUSIC_GENRES`, em `db-types/src/organization.rs`, e a mesma lista existe em `app/src/pages/create-organization.schema.ts`. **As duas precisam bater**: divergindo, o formulário aceita e a API recusa. A lista é fechada porque é ela que sustenta o filtro do mapa — texto livre viraria "techno", "Techno" e "tekno" como três coisas diferentes.

## Pedidos de correção

Quem vê um dado errado no site sugere a correção sem ter conta:

```http
POST /organizations/{id_ou_slug}/edit-requests
{ "changes": { "city": "Campinas" }, "message": "mudaram de cidade", "requester_email": "quem@sugeriu.com" }
```

`changes` aceita qualquer subconjunto dos campos editáveis; o que não vier fica como está. A sugestão **não altera nada** — entra numa fila e só vale quando a moderação aplica. Aplicar grava `reviewed_by` e `reviewed_at`, e o cadastro continua no estado em que estava.

Editar é só da moderação. A porta aberta é a de sugerir.

## Moderação

Cadastro criado pelo site nasce `pending` e **não aparece nas rotas públicas**. `GET /organizations` e `GET /organizations/{id}` só enxergam `approved` — um cadastro pendente responde 404, e não "existe mas está escondido".

Os três estados são `pending`, `approved` e `rejected`, garantidos por um CHECK no banco.

Rejeitar apaga os arquivos do bucket, e por isso é definitivo: aprovar depois produziria um cadastro sem imagem nenhuma. O motivo é custo — um envio automatizado empurra dezenas de megabytes por vez. As linhas do banco ficam, com o motivo da recusa.

Aprovação e rejeição gravam `reviewed_at` e `reviewed_by`.

### Criando o primeiro moderador

Não há rota pública para criar admin. Há dois caminhos.

**Pelo ambiente**, que é o que serve para deploy sem console:

```bash
ADMIN_NAME=Moderação
ADMIN_EMAIL=voce@dominio.com
ADMIN_PASSWORD=            # mínimo 12 caracteres
```

A API cria a conta na subida **apenas quando não existe nenhum moderador**. É semente de primeira subida, não fonte permanente de senha: se aplicasse a senha a cada boot, quem tivesse acesso ao painel assumiria a conta editando a variável e reiniciando o serviço. Nas subidas seguintes ela é ignorada e o log avisa para removê-la — e vale remover mesmo, porque variável de ambiente aparece em `docker inspect` e em `/proc/<pid>/environ`.

Senha menor que 12 caracteres **derruba a subida**, em vez de criar uma conta fraca em silêncio.

**Pelo binário**, para criar moderadores depois do primeiro:

```bash
docker compose exec rust-api create_admin "Nome" email@dominio.com
```

A senha vem pela entrada padrão, nunca por argumento — argumento de processo aparece na lista de processos e no histórico do shell. Funciona em pipe: `echo "senha" | create_admin ...`.

Senha guardada com argon2. E-mail inexistente e senha errada devolvem a mesma mensagem e gastam o mesmo tempo, de propósito: a diferença revelaria quais e-mails existem.

## Autorização: a prova mora no tipo

A regra que sustenta o site é que cadastro pendente ou rejeitado não existe para quem está de fora. Ela poderia morar só na escolha de qual função do repositório o handler chama — `find_approved_by_id` filtra por status, `find_by_id` não. As duas têm a mesma assinatura e nomes parecidos, e nada impediria uma rota pública de chamar a errada.

Então quem enxerga qualquer estado exige uma prova, `SeeEveryStatus`, que só pode ser construída em `domains/organizations/auth.rs` e só a partir de um moderador autenticado. Rota pública não tem como produzir uma, então **não compila**.

A prova é um extractor: o handler pede o direito de que precisa, e a autenticação, o 401 e a decisão inteira ficam no `auth.rs`.

```rust
#[get("/admin/organizations")]
pub async fn moderation_index(
    w: SeeEveryStatus,
    ...
```

O que isso fecha e o que não fecha: fecha o acidente de uma rota pública chamar a query errada. Não é barreira contra quem fabrica a identidade na mão — é assim que os testes montam a sua. Fechar isso protegeria contra um ato deliberado, que apareceria na revisão de qualquer jeito.

## Imagens

Ficam num bucket compatível com S3, nunca no disco do container — disco de container é efêmero e não é compartilhado entre réplicas.

O upload é validado por *magic bytes* antes de subir: o formato sai do conteúdo, não do nome nem do content-type que o cliente mandou, e SVG é recusado por ser documento que executa script. O nome do objeto é sorteado, o content-type gravado vem da detecção, e envio que falha no meio tem os objetos já subidos apagados.

O banco guarda **só a chave do objeto**; a URL é montada na resposta a partir de `S3_PUBLIC_BASE_URL`. Trocar de provedor ou pôr um CDN na frente não exige migração de dados.

Em produção a política do bucket deve permitir apenas `s3:GetObject` em `arn:aws:s3:::<bucket>/*`. Liberar `ListBucket` expõe o nome de toda imagem enviada, inclusive de cadastros que a moderação nunca aprovou.

## Healthcheck

| Rota | Responde | Verifica |
|---|---|---|
| `GET /health` | 200 sempre que o processo está de pé | nada externo |
| `GET /health/ready` | 200 ou **503** | banco e bucket, com teto de 3s cada |

```json
// GET /health/ready com o bucket fora do ar
{ "status": "unavailable", "database": "ok", "storage": { "error": "tempo esgotado" } }
```

Use `/health` para política de **reinício**: derrubar o container porque o Postgres piscou não conserta nada e ainda tira a API do ar junto. Use `/health/ready` para decidir **roteamento de tráfego**.

No EasyPanel, aponte para `/health/ready` se ele apenas marcar o serviço como indisponível; se ele reiniciar o container ao falhar, aponte para `/health`.

A resposta diz **qual** dependência falhou, de propósito: sonda que só responde "não" obriga quem está de plantão a adivinhar.

## Testes

```bash
cargo test
```

**Cuidado:** os testes que precisam de Postgres se anunciam como pulados quando `TEST_DATABASE_URL` não está definida, e o `cargo test` termina verde do mesmo jeito. São 53 dos 173 — a moderação inteira, os pedidos de correção, o contrato HTTP e o acesso administrativo. Para rodar tudo de verdade:

```bash
docker compose up -d database
docker compose exec database psql -U docker -d postgres -c "CREATE DATABASE rave_map_test;"

TEST_DATABASE_URL=postgres://docker:ravemap@localhost:5432/rave_map_test cargo test
```

A variável é separada de `DATABASE_URL` de propósito, e apontar as duas para o mesmo lugar é recusado em tempo de execução: estes testes apagam tabela para montar o cenário.

```bash
cargo clippy --all-targets
cargo fmt
```

## Migrações

Ficam em `migrations/` e são **embutidas no binário**, então o deploy não precisa do CLI do Diesel. Para criar uma:

```bash
diesel migration generate nome_da_mudanca
diesel migration run       # aplica e regenera db-types/src/schema.rs
diesel migration revert
```

## Deploy

O `Dockerfile` está na **raiz do repositório**, e é lá que o contexto de build precisa apontar: a API é um crate de um workspace e o build usa o `Cargo.toml` da raiz mais `api-types` e `db-types`. No EasyPanel o campo é **Build path**; deixe em `/`. Apontar para `/api-rust` falha com `COPY api-types api-types: "/api-types": not found`.

A imagem é Debian slim, não Alpine, porque o Diesel linka com `libpq`.

O processo roda como uid 10001 e escuta em 8080. Se o painel proxeia para a porta 80, o serviço responde 502 — não é possível ligar em porta abaixo de 1024 sem root.
