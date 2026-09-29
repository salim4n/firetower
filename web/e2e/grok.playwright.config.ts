import { defineConfig } from "@playwright/test";

// Run against a real Firetower server and the matching desktop renderer:
// playwright test --config e2e/grok.playwright.config.ts
export default defineConfig({
  testDir: ".",
  testMatch: "grok-control-plane.spec.ts",
  workers: 1,
  use: {
    headless: true,
    // The sign-in test handles a live device code and an account password.
    // Browser traces and failure screenshots must not become CI artifacts.
    screenshot: "off",
    trace: "off",
  },
});
