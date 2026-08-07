import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import reactRefresh from "eslint-plugin-react-refresh";
import prettier from "eslint-plugin-prettier";
import prettierConfig from "eslint-config-prettier";
import { defineConfig } from "eslint/config";

export default defineConfig(
  {
    ignores: ["dist", "node_modules", ".vscode", "vite.config.ts", "*.config.*"],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        project: ["./tsconfig.json"],
      },
    },
    plugins: {
      "react-refresh": reactRefresh,
      prettier,
    },
    rules: {
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/no-explicit-any": "warn",
      "prettier/prettier": "warn",
      "react/prop-types": "off", // Using TypeScript for prop validation
    },
    settings: {
      react: {
        version: "18.2",
      },
    },
  },
  prettierConfig
);
