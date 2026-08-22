# O contexto de build é esta pasta (app/), não a raiz do repositório: o
# front-end não depende de nada fora daqui.
#
#   docker build -t rave-map-app app/
#   EasyPanel: Build path "/app"

# ---------- build ----------
FROM node:22-alpine AS builder

WORKDIR /build

# As variáveis do Vite são resolvidas na COMPILAÇÃO, não na execução: elas
# viram texto dentro do bundle. Por isso chegam como ARG, e por isso não
# adianta configurá-las como variável de ambiente do container em produção.
#
# Todas são públicas por natureza — qualquer visitante lê o bundle. O token do
# Mapbox se protege restringindo os domínios no painel deles, não escondendo.
# Segredo de verdade (JWT_SECRET, senha de banco) vive na API e nunca aqui.
ARG VITE_MAPBOX_USERNAME
ARG VITE_MAPBOX_STYLE_ID
ARG VITE_MAPBOX_ACCESS_TOKEN
ARG VITE_SITE_URL
ARG VITE_API_URL
ARG VITE_IMAGES_BASE_URL

ENV VITE_MAPBOX_USERNAME=$VITE_MAPBOX_USERNAME \
    VITE_MAPBOX_STYLE_ID=$VITE_MAPBOX_STYLE_ID \
    VITE_MAPBOX_ACCESS_TOKEN=$VITE_MAPBOX_ACCESS_TOKEN \
    VITE_SITE_URL=$VITE_SITE_URL \
    VITE_API_URL=$VITE_API_URL \
    VITE_IMAGES_BASE_URL=$VITE_IMAGES_BASE_URL

# Dependências em camada própria: só reinstala quando os manifestos mudam.
COPY package.json package-lock.json ./
RUN npm ci

COPY . .

# `npm run build` roda o tsc antes do vite: erro de tipo derruba o deploy em
# vez de virar bug em produção.
RUN npm run build

# ---------- runtime ----------
FROM nginx:alpine

# Configuração de SPA: qualquer caminho cai no index.html, senão abrir
# /raves direto na barra de endereço responde 404 do servidor.
COPY nginx.conf /etc/nginx/conf.d/default.conf

COPY --from=builder /build/dist /usr/share/nginx/html

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s \
    CMD wget -q --spider http://127.0.0.1/ || exit 1
