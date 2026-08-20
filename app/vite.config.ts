import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react-swc";
import tailwindcss from "@tailwindcss/vite";
import tsconfigPaths from "vite-tsconfig-paths";
import { z } from "zod";

const envSchema = z.object({
  VITE_USERNAME: z.string().min(1, "VITE_USERNAME is required"),
  VITE_STYLE_ID: z.string().min(1, "VITE_STYLE_ID is required"),
  VITE_ACCESS_TOKEN: z.string().min(1, "VITE_ACCESS_TOKEN is required"),
});

const DEFAULT_SITE_URL = "http://localhost:5173";

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

  if (mode === "production" && !env.VITE_SITE_URL) {
    console.warn(
      "⚠️  VITE_SITE_URL nao definida: canonical, Open Graph e sitemap vao apontar para " +
        DEFAULT_SITE_URL
    );
  }

  return {
    base: "/",
    plugins: [react(), tailwindcss(), tsconfigPaths(), seoFiles(siteUrl)],
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
