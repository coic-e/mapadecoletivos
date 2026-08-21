import { createHash } from "node:crypto";

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

const DEFAULT_SITE_URL = "http://localhost:5173";

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
function csp(apiUrl: string, inlineScriptHashes: string[]) {
  const mapbox = "https://api.mapbox.com https://*.tiles.mapbox.com https://events.mapbox.com";

  return [
    "default-src 'self'",
    `script-src 'self' ${inlineScriptHashes.join(" ")}`.trim(),
    // Leaflet e o React escrevem style inline nos elementos do mapa; o
    // 'unsafe-inline' vale para atributo style, não para <script>.
    "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
    "font-src 'self' https://fonts.gstatic.com",
    `img-src 'self' data: blob: ${[apiUrl, mapbox].filter(Boolean).join(" ")}`,
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
function cspMeta(apiUrl: string) {
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
          `  <meta http-equiv="Content-Security-Policy" content="${csp(apiUrl, hashes)}" />\n  </head>`
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

  if (mode === "production" && apiUrl === "") {
    console.warn("⚠️  VITE_API_URL vazia: build em modo de demonstração, com dados estáticos");
  }

  if (mode === "production" && !env.VITE_SITE_URL) {
    console.warn(
      "⚠️  VITE_SITE_URL nao definida: canonical, Open Graph e sitemap vao apontar para " +
        DEFAULT_SITE_URL
    );
  }

  return {
    base: "/",
    plugins: [react(), tailwindcss(), tsconfigPaths(), seoFiles(siteUrl), cspMeta(apiUrl)],
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
