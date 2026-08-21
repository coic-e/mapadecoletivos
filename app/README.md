# Mapa de Rave — front-end

Mapa interativo dos coletivos, festas, labels e clubs de música eletrônica do Brasil. Parte do monorepo [mapadecoletivos](../README.md).

## Stack

| Camada | Ferramenta |
|---|---|
| Build | Vite 7 + `@vitejs/plugin-react-swc` |
| UI | React 19 + TypeScript |
| Estilo | Tailwind v4 + componentes shadcn em `src/components/ui` |
| Mapa | Leaflet + react-leaflet, tiles do Mapbox |
| Formulário | react-hook-form + zod |
| Testes | Vitest + Testing Library |

Não há arquivo de CSS por página: tudo é Tailwind, e `src/styles/tailwind.css` guarda só os tokens de cor em `:root` e `.dark`. CSS de terceiros (leaflet) é importado dentro de `layer(base)` para que os utilitários do Tailwind continuem vencendo dele.

## Rodando

Requer Node 20+ e a API do monorepo no ar (`docker compose up -d` na raiz).

```bash
cp .env.example .env
npm install
npm start
```

O site sobe em `http://localhost:5173` e fala com a API em `http://localhost:8080`.

## Variáveis de ambiente

`.env` é validado por zod na subida — se faltar alguma obrigatória, o Vite aborta com a lista do que falta em vez de quebrar em runtime.

| Variável | Obrigatória | Para quê |
|---|---|---|
| `VITE_MAPBOX_USERNAME` | sim | usuário do Mapbox, monta a URL dos tiles |
| `VITE_MAPBOX_STYLE_ID` | sim | id do estilo do mapa no Mapbox |
| `VITE_MAPBOX_ACCESS_TOKEN` | sim | token público do Mapbox: tiles e busca de endereço |
| `VITE_SITE_URL` | em produção | URL absoluta do site; alimenta canonical, Open Graph, `robots.txt` e `sitemap.xml` |
| `VITE_API_URL` | não | endereço da API; **vazia liga o modo de demonstração** |

O token do Mapbox vai no bundle e é visível para qualquer pessoa que abrir o
site — é assim que funciona mapa no navegador. O que impede o abuso não é
escondê-lo, e sim restringi-lo por URL no painel do Mapbox, para ele só
responder a partir dos seus domínios. Ele é público de propósito; o segredo que
não pode vazar é o `JWT_SECRET`, que vive na API e nunca chega ao front.

Sem `VITE_SITE_URL`, o build de produção avisa no console e cai em `http://localhost:5173` — o que faz o Google indexar URLs de localhost. Defina no ambiente de deploy.

## Modo de demonstração

Com `VITE_API_URL` **vazia**, o app não chama API nenhuma: um adapter do axios responde com os dados estáticos de `src/services/seed/`. Serve para publicar o front sozinho — numa preview da Vercel, por exemplo — e revisar telas antes de a API existir num servidor.

```bash
VITE_API_URL=            # vazio liga a demonstração
```

O que muda:

- sete coletivos inventados aparecem no mapa, um deles pendente para o painel de moderação ter o que mostrar;
- qualquer e-mail e senha entram em `/admin/login`, porque não há segredo a proteger em dado inventado;
- cadastrar, sugerir correção, aprovar e rejeitar funcionam **na memória da aba** e somem no reload;
- uma faixa avisa, em toda tela, que os dados são de demonstração.

As capas são SVGs em `public/seed/`, e os e-mails usam o domínio reservado `example.com`, que nunca chega a ninguém.

Isto não substitui o backend para teste sério: nada é validado de verdade, e o adapter só cobre as rotas que as telas usam. As credenciais do Mapbox continuam obrigatórias, senão o mapa sobe sem tiles.

## Scripts

```bash
npm start          # dev server
npm run build      # tsc + build de produção em dist/
npm run serve      # serve o dist/ localmente
npm test           # vitest em watch
npm run lint       # eslint, zero warnings tolerados
npm run format     # prettier
```

`npm start -- --host` expõe na rede local (repare no `--`, senão o npm engole a flag).

## Estrutura

```
src/
├── components/
│   ├── ui/          # componentes shadcn (button, input, select, form...)
│   ├── CityScape/   # cidade procedural em WebGL2 do fundo da home
│   └── Sidebar/
├── config/env.ts    # variáveis validadas com zod — use isto, não import.meta.env
├── hooks/useSeo.ts  # título, description e canonical por rota
├── pages/           # Landing, OrganizationsMap, Organization, CreateOrganization
├── services/        # api (axios) e geocode (Mapbox)
└── styles/          # tailwind.css: só tokens
```

## Rotas

| Rota | Página |
|---|---|
| `/` | Landing com a cidade em WebGL |
| `/raves` | Mapa do Brasil com todos os cadastros |
| `/raves/:id` | Página de um rolê |
| `/raves/create` | Formulário de cadastro |
| `/admin/login` | Login dos moderadores |
| `/admin` | Fila de moderação |

Cadastro novo entra como pendente e só aparece no mapa depois que um moderador aprova em `/admin`. Para criar a primeira conta de moderador, veja o [README da API](../api-rust/README.md#criando-o-primeiro-moderador).

## SEO

`index.html` carrega as meta tags estáticas e `useSeo` ajusta título, descrição e canonical por rota. `robots.txt` e `sitemap.xml` são gerados no build por um plugin do Vite, a partir de `VITE_SITE_URL` — por isso não estão em `public/`, onde arquivos são copiados sem substituição de variável.

**Limitação conhecida:** o app é uma SPA sem SSR. O Google executa JS e indexa normalmente, mas os robôs de preview (WhatsApp, Instagram, Facebook, Twitter) não executam — eles leem só o `index.html`. Compartilhar o link de um rolê específico mostra o preview genérico da home até que as rotas sejam pré-renderizadas. Falta também um `public/og-cover.png` de 1200×630, referenciado pelas tags Open Graph.

## Contribuindo

1. Faça um fork e crie sua branch (`git checkout -b feat/nome`)
2. Antes do PR: `npm run lint && npx tsc --noEmit && npx vitest run && npm run build`
3. Abra o Pull Request

## Licença

MIT, herdada do monorepo. O arquivo `LICENSE` ainda não existe — veja o [README da raiz](../README.md).
