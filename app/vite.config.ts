import { defineConfig, loadEnv } from 'vite'
import react from '@vitejs/plugin-react-swc'
import { z } from 'zod'

const envSchema = z.object({
  VITE_USERNAME: z.string().min(1, "VITE_USERNAME is required"),
  VITE_STYLE_ID: z.string().min(1, "VITE_STYLE_ID is required"),
  VITE_ACCESS_TOKEN: z.string().min(1, "VITE_ACCESS_TOKEN is required"),
});

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), 'VITE_')
  
  // Validate environment variables at startup
  try {
    envSchema.parse(env);
    console.log('✅ Environment variables validated successfully');
  } catch (error) {
    if (error instanceof z.ZodError) {
      const missingVars = error.issues
        .map((err) => `  - ${err.path.join(".")}: ${err.message}`)
        .join("\n");
      console.error('\n❌ Invalid environment variables:\n' + missingVars);
      console.error('\nPlease check your .env file and ensure all required variables are set.\n');
      process.exit(1);
    }
    throw error;
  }

  return {
    base: '/',
    plugins: [react()],
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: './src/setupTests.ts',
      css: true,
      reporters: ['verbose'],
      coverage: {
        reporter: ['text', 'json', 'html'],
        include: ['src/**/*'],
        exclude: [],
      }
    },
  }
})