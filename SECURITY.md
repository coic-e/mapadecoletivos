# Segurança

O que a aplicação já faz por conta própria, o que ela exige de quem faz o
deploy, e o que continua em aberto.

## O que a API garante sozinha

| Defesa | Onde |
| --- | --- |
| Senha de moderador com argon2 e salt por conta | `api-rust/src/auth/mod.rs` |
| Token JWT com algoritmo fixo (HS256), `exp` obrigatório e conta conferida no banco a cada requisição | `api-rust/src/auth/mod.rs` |
| Login com resposta e tempo idênticos para e-mail inexistente e senha errada | `api-rust/src/domains/admins/routes.rs` |
| Rate limit por IP no login e no cadastro público | `api-rust/src/rate_limit.rs` |
| Upload só de JPEG, PNG, GIF e WebP, conferidos pelos bytes do arquivo | `api-rust/src/handlers/upload.rs` |
| Nome de arquivo sorteado no servidor, nunca vindo do cliente | `api-rust/src/handlers/upload.rs` |
| `/uploads` sem listagem, com `nosniff` e CSP `sandbox` | `api-rust/src/handlers/static_files.rs` |
| Tetos de tamanho: por arquivo, por campo, por requisição e por página | `api-rust/src/config.rs` |
| Erro interno vira mensagem genérica; o detalhe fica no log | `api-rust/src/errors/api_error.rs` |
| Recusa subir com `JWT_SECRET` curto, de exemplo, ou com `CORS_ALLOWED_ORIGINS=*` | `api-rust/src/config.rs` |

O site complementa com uma CSP gerada no build (`app/vite.config.ts`) e os
cabeçalhos de `app/public/_headers`.

## O que o deploy precisa fornecer

1. **`JWT_SECRET` com 32+ caracteres, gerado aleatoriamente.** `openssl rand
   -base64 48`. A API recusa subir com os segredos de exemplo do repositório.
   Trocar o segredo derruba todas as sessões abertas — é o botão de emergência
   se um token vazar.
2. **`CORS_ALLOWED_ORIGINS` com o domínio do site**, separado por vírgula se
   houver mais de um. Sem isso vale o padrão de desenvolvimento
   (`http://localhost:5173`) e o site em produção não consegue chamar a API.
3. **HTTPS na frente de tudo.** O token do moderador vai no cabeçalho
   `Authorization`; em HTTP ele viaja legível para qualquer um no caminho. O
   site também recusa build com `VITE_API_URL` em http fora de localhost.
4. **`TRUST_PROXY=true` apenas se houver um proxy reverso** que reescreva
   `X-Forwarded-For`. Ligado sem proxy, qualquer cliente forja o próprio IP e
   escapa do rate limit; desligado atrás de um proxy, todos os visitantes
   contam como um IP só e travam uns aos outros.
5. **Os cabeçalhos de `app/public/_headers` servidos de verdade.** Netlify e
   Cloudflare Pages leem o arquivo direto. Em Nginx:

   ```nginx
   add_header X-Frame-Options DENY always;
   add_header Content-Security-Policy "frame-ancestors 'none'" always;
   add_header X-Content-Type-Options nosniff always;
   add_header Referrer-Policy strict-origin-when-cross-origin always;
   add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
   ```

6. **Postgres fora da internet.** O `docker-compose.yml` publica a porta só em
   `127.0.0.1`; um banco gerenciado deve ficar em rede privada.

## Contas de moderador

Não existe rota de cadastro. A conta nasce pelo binário, com a senha lida da
entrada padrão — nunca como argumento, que apareceria no `ps` e no histórico
do shell:

```bash
cargo run --bin create_admin -- "Nome da Pessoa" pessoa@exemplo.com
# senha do moderador: (digitada aqui)
```

Mínimo de 12 caracteres. Revogar é apagar a linha da tabela `admins`: o
extractor confere a conta no banco a cada requisição, então o acesso cai na
requisição seguinte, sem esperar o token expirar.

## Riscos aceitos

- **O token do moderador fica no `localStorage`.** Um XSS no site o levaria
  embora. A alternativa — cookie `httpOnly` — troca esse risco por CSRF e exige
  cookie cross-site com `SameSite=None`, já que site e API vivem em domínios
  diferentes. A escolha foi manter o Bearer token (imune a CSRF por
  construção) e fechar o XSS pela CSP. Se um dia site e API ficarem no mesmo
  domínio, o cookie `httpOnly` com `SameSite=Strict` passa a ser a opção
  melhor.
- **O rate limit é por processo.** Vale para uma instância. Com várias
  réplicas, o contador precisa sair para um Redis.
- **O e-mail do coletivo é público**, porque a página oferece o contato. Vale
  saber que robôs de spam vão colhê-lo.
- **`h2 0.3` carrega o RUSTSEC-2026-0258**, que só afeta HTTP/2. O actix-web 4
  ainda não migrou para o `h2 0.4`, e a API serve HTTP/1.1 em texto puro atrás
  do proxy — o caminho vulnerável não é alcançável nessa topologia. Revisar
  quando o actix-web atualizar.

## Verificando

```bash
cargo test --workspace     # inclui os testes das defesas acima
cargo audit                # dependências Rust
npm audit --omit=dev       # dependências do site (em app/)
```
