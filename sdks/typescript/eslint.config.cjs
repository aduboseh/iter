const js = require("@eslint/js");
const { defineConfig } = require("eslint/config");
const tseslint = require("typescript-eslint");

module.exports = defineConfig(
  {
    files: ["src/**/*.ts"],
    extends: [js.configs.recommended, tseslint.configs.recommended],
  },
  {
    files: ["src/**/*.test.ts"],
    rules: {
      // Existing process doubles exercise private client state and event handlers.
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unsafe-function-type": "off",
    },
  },
);
