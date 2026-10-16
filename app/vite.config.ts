import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react-swc";
import tailwindcss from "@tailwindcss/vite";
import tsconfigPaths from "vite-tsconfig-paths";
import { z } from "zod";

const envSchema = z.object({
  VITE_MAPBOX_USERNAME: z.string().min(1, "VITE_MAPBOX_USERNAME is required"),
  VITE_MAPBOX_STYLE_ID: z.string().min(1, "VITE_MAPBOX_STYLE_ID is required"),
  VITE_MAPBOX_ACCESS_TOKEN: z.string().min(1, "VITE_MAPBOX_ACCESS_TOKEN is required"),
});

// O domínio de produção, e não localhost, porque este valor só alimenta
// canonical, Open Graph, robots.txt e sitemap.xml. Um build que perdeu a
// variável passa a anunciar o endereço certo em vez de mandar o Google indexar
// URLs de localhost — e em desenvolvimento nada disso é lido por robô nenhum.
const DEFAULT_SITE_URL = "https://mapaderave.com.br";

/**
 * Content-Security-Policy do site.
 *
 * É a rede de proteção contra XSS: mesmo que algum texto de cadastro escape
 * para o HTML, o navegador se recusa a executar script que não venha do
 * próprio domínio. Importa aqui porque o token do moderador vive no
 * localStorage — um script injetado que rodasse o levaria embora.
 *
 * O bloco JSON-LD do index.html é inline e muda com VITE_SITE_URL, então o
 * hash dele é calculado no build em vez de ficar escrito à mão.
 */
/**
 * `imagesUrl` é a origem do bucket, separada da API de propósito: as fotos dos
 * cadastros são servidas pelo bucket, e sem ela no img-src o navegador bloqueia
 * todas as imagens do site.
 */
function csp(apiUrl: string, imagesUrl: string, inlineScriptHashes: string[]) {
  const mapbox = "https://api.mapbox.com https://*.tiles.mapbox.com https://events.mapbox.com";

  return [
    "default-src 'self'",
    `script-src 'self' ${inlineScriptHashes.join(" ")}`.trim(),
    // Leaflet e o React escrevem style inline nos elementos do mapa; o
    // 'unsafe-inline' vale para atributo style, não para <script>.
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com",
    `img-src 'self' data: blob: ${[apiUrl, imagesUrl, mapbox].filter(Boolean).join(" ")}`,
    `connect-src 'self' ${[apiUrl, mapbox].filter(Boolean).join(" ")}`,
    // Nada de plugin, nada de <base> reescrito, nada de iframe.
    "object-src 'none'",
    "base-uri 'none'",
    "frame-src 'none'",
    "form-action 'self'",
    // Ignorado quando vem por <meta>; o cabeçalho equivalente está em
    // public/_headers, para o deploy servir de verdade.
    "frame-ancestors 'none'",
    // Só quando a API já é https: com uma API em http (dev, preview local) a
    // diretiva promoveria a chamada para https e quebraria o site.
    ...(apiUrl.startsWith("https://") ? ["upgrade-insecure-requests"] : []),
  ].join("; ");
}

/**
 * Injeta a CSP no index.html depois que o Vite já substituiu as %VITE_*%,
 * para que o hash do JSON-LD bata com o que vai ao ar.
 */
function cspMeta(apiUrl: string, imagesUrl: string) {
  return {
    name: "csp-meta",
    // Só no build: o dev server recarrega módulo por módulo com script inline
    // gerado a cada troca, e nenhum hash fixo daria conta.
    apply: "build" as const,
    transformIndexHtml: {
      order: "post" as const,
      handler(html: string) {
        const hashes = [...html.matchAll(/<script(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g)].map(
          (match) => `'sha256-${createHash("sha256").update(match[1], "utf8").digest("base64")}'`
        );

        return html.replace(
          "</head>",
          `  <meta http-equiv="Content-Security-Policy" content="${csp(apiUrl, imagesUrl, hashes)}" />\n  </head>`
        );
      },
    },
  };
}

/**
 * robots.txt e sitemap.xml precisam da URL absoluta do site, e arquivos em
 * public/ sao copiados sem substituicao de variavel: so o index.html passa
 * pelo replace de %VITE_*%. Por isso os dois sao emitidos no build, a partir
 * da mesma env, para o dominio nao ficar escrito em dois lugares.
 */
function seoFiles(siteUrl) {
  return {
    name: "seo-files",

    // O replace de %VITE_SITE_URL% do Vite usa o valor cru da variável. Se ela
    // terminar em barra — e terminar é o normal quando se copia da barra de
    // endereço — sai `https://site//`, e canonical apontando para um endereço
    // que não é o real conta como conteúdo duplicado. Aqui a URL já vem
    // normalizada, e esta substituição roda antes da do Vite, que então não
    // acha mais nada para trocar.
    transformIndexHtml: {
      order: "pre",
      handler(html) {
        return html.replaceAll("%VITE_SITE_URL%", siteUrl);
      },
    },

    generateBundle() {
      this.emitFile({
        type: "asset",
        fileName: "robots.txt",
        source: [
          "User-agent: *",
          "Allow: /",
          "",
          "# Formulario de cadastro nao tem valor de busca",
          "Disallow: /raves/create",
          "",
          "# Area de moderacao",
          "Disallow: /admin",
          "",
          `Sitemap: ${siteUrl}/sitemap.xml`,
          "",
        ].join("\n"),
      });

      this.emitFile({
        type: "asset",
        fileName: "sitemap.xml",
        source: [
          '<?xml version="1.0" encoding="UTF-8"?>',
          '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
          `  <url><loc>${siteUrl}/</loc><changefreq>monthly</changefreq><priority>1.0</priority></url>`,
          `  <url><loc>${siteUrl}/raves</loc><changefreq>daily</changefreq><priority>0.9</priority></url>`,
          "</urlset>",
          "",
        ].join("\n"),
      });
    },

    // A imagem de compartilhamento vinha apontando para um arquivo que nunca
    // existiu, e nada acusava: quem abre o site não percebe, o build passa, e
    // a falha só aparece quando alguém cola o link em algum lugar. Como o
    // build é o portão do deploy, é aqui que isso tem que gritar.
    closeBundle() {
      const html = readFileSync(resolve(__dirname, "dist/index.html"), "utf8");
      const referencia = html.match(/property="og:image" content="([^"]+)"/);

      if (!referencia) {
        this.error("index.html não declara og:image");
      }

      const arquivo = referencia[1].replace(siteUrl, "");

      if (!existsSync(resolve(__dirname, "dist", arquivo.replace(/^\//, "")))) {
        this.error(
          `og:image aponta para ${arquivo}, que não existe no build. ` +
            "O arquivo precisa estar em public/."
        );
      }
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), "VITE_");
  // Validate environment variables at startup
  try {
    envSchema.parse(env);
    console.log("✅ Environment variables validated successfully");
  } catch (error) {
    if (error instanceof z.ZodError) {
      const missingVars = error.issues
        .map((err) => `  - ${err.path.join(".")}: ${err.message}`)
        .join("\n");
      console.error("\n❌ Invalid environment variables:\n" + missingVars);
      console.error("\nPlease check your .env file and ensure all required variables are set.\n");
      process.exit(1);
    }
    throw error;
  }

  const siteUrl = (env.VITE_SITE_URL || DEFAULT_SITE_URL).replace(/\/$/, "");
  // Ausente significa "não há API": o app roda com dados de demonstração e o
  // CSP não precisa liberar origem nenhuma para chamadas.
  const apiUrl = (env.VITE_API_URL ?? "").trim().replace(/\/$/, "");

  // Só a origem interessa ao CSP: o caminho do bucket dentro dela é ignorado.
  const imagesUrl = (() => {
    const raw = (env.VITE_IMAGES_BASE_URL ?? "").trim();

    if (raw === "") return "";

    try {
      return new URL(raw).origin;
    } catch {
      console.warn("⚠️  VITE_IMAGES_BASE_URL não é uma URL absoluta; ignorada no CSP");

      return "";
    }
  })();

  if (mode === "production" && apiUrl === "") {
    console.warn("⚠️  VITE_API_URL vazia: build em modo de demonstração, com dados estáticos");
  }

  if (mode === "production" && !env.VITE_SITE_URL) {
    console.warn(`⚠️  VITE_SITE_URL nao definida: usando ${DEFAULT_SITE_URL}`);
  }

  return {
    base: "/",
    plugins: [
      react(),
      tailwindcss(),
      tsconfigPaths(),
      seoFiles(siteUrl),
      cspMeta(apiUrl, imagesUrl),
    ],
    test: {
      globals: true,
      environment: "jsdom",
      setupFiles: "./src/setupTests.ts",
      css: true,
      reporters: ["verbose"],
      coverage: {
        reporter: ["text", "json", "html"],
        include: ["src/**/*"],
        exclude: [],
      },
    },
  };
});
