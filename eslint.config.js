// eslint.config.js
import { defineConfig } from "eslint/config";

export default defineConfig([
  {
    files: ["**/*.js"],
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module"
    },
    rules: {
      semi: "error",
      "prefer-const": "error",
      "no-unused-vars": "off"
    }
  }
]);
