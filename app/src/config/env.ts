import { z } from "zod";

/**
 * Environment variable schema validation using Zod
 * Ensures all required environment variables are present and valid
 */
const envSchema = z.object({
  VITE_USERNAME: z.string().min(1, "VITE_USERNAME is required"),
  VITE_STYLE_ID: z.string().min(1, "VITE_STYLE_ID is required"),
  VITE_ACCESS_TOKEN: z.string().min(1, "VITE_ACCESS_TOKEN is required"),
  /** URL absoluta do site, usada em canonical e Open Graph. Opcional em dev. */
  VITE_SITE_URL: z.string().default(""),
  /**
   * URL da API. Precisa ser https fora de localhost: em página https, uma API
   * http é bloqueada pelo navegador como conteúdo misto — e, quando passa, o
   * token do moderador viaja em texto claro.
   */
  VITE_API_URL: z
    .url("VITE_API_URL precisa ser uma URL absoluta")
    .refine(
      (value) => value.startsWith("https://") || new URL(value).hostname === "localhost",
      "VITE_API_URL precisa usar https fora de localhost"
    )
    .default("http://localhost:8080"),
});

type EnvSchema = z.infer<typeof envSchema>;

/**
 * Validates and returns typed environment variables
 * Throws an error if validation fails
 */
function validateEnv(): EnvSchema {
  try {
    return envSchema.parse({
      VITE_USERNAME: import.meta.env.VITE_USERNAME,
      VITE_STYLE_ID: import.meta.env.VITE_STYLE_ID,
      VITE_ACCESS_TOKEN: import.meta.env.VITE_ACCESS_TOKEN,
      VITE_SITE_URL: import.meta.env.VITE_SITE_URL,
      VITE_API_URL: import.meta.env.VITE_API_URL,
    });
  } catch (error) {
    if (error instanceof z.ZodError) {
      const missingVars = error.issues
        .map((err) => `${err.path.join(".")}: ${err.message}`)
        .join("\n");
      throw new Error(
        `❌ Invalid environment variables:\n${missingVars}\n\nPlease check your .env file and ensure all required variables are set.`,
        { cause: error }
      );
    }
    throw error;
  }
}

/**
 * Validated and typed environment variables
 * Use this instead of import.meta.env directly
 */
export const env = validateEnv();
