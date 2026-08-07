import { z } from "zod";

/**
 * Environment variable schema validation using Zod
 * Ensures all required environment variables are present and valid
 */
const envSchema = z.object({
  VITE_USERNAME: z.string().min(1, "VITE_USERNAME is required"),
  VITE_STYLE_ID: z.string().min(1, "VITE_STYLE_ID is required"),
  VITE_ACCESS_TOKEN: z.string().min(1, "VITE_ACCESS_TOKEN is required"),
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
