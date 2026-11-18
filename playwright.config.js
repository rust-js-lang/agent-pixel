import { defineConfig } from "@playwright/test";

const port = 4187;
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  use: {
    baseURL: `http://127.0.0.1:${port}`,
    launchOptions: { args: ["--enable-features=WebMCP"] },
    trace: "retain-on-failure"
  },
  webServer: {
    command: "bun run build && cargo build --locked --manifest-path server/Cargo.toml && bun e2e/serve.js",
    env: { AGENT_PIXEL_PORT: String(port) },
    url: `http://127.0.0.1:${port}/api/document`,
    timeout: 120_000
  }
});
