import { readFileSync, writeFileSync } from "node:fs";
import { expect, test, type Page } from "@playwright/test";

const desktop = process.env.FIRETOWER_E2E_DESKTOP_URL;
const server = process.env.FIRETOWER_E2E_SERVER;
const passwordFile = process.env.FIRETOWER_E2E_PASSWORD_FILE;

async function signIn(page: Page) {
  if (!desktop || !server || !passwordFile) throw new Error("E2E environment is incomplete");
  await page.goto(desktop);
  await page.getByPlaceholder("ft-e1.tail9c2b.ts.net").fill(server);
  await page.getByRole("button", { name: "Connect", exact: true }).click();
  await page.getByPlaceholder("Username").fill("admin");
  await page.getByPlaceholder("Password").fill(readFileSync(passwordFile, "utf8"));
  await page.getByRole("button", { name: "Sign in", exact: true }).click();
  await expect(page.getByRole("button", { name: "Configuration" })).toBeVisible();
  await page.getByRole("button", { name: "Configuration" }).click();
  await page.getByRole("button", { name: /Grok Build/ }).click();
}

test.describe("Grok Build on a fresh Firetower worker", () => {
  test.skip(!desktop || !server || !passwordFile, "set the real desktop URL, server, and password file");

  test("installs the pinned CLI from the control plane", async ({ page }) => {
    test.setTimeout(180_000);
    await signIn(page);
    await expect(page.getByText("not installed", { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Install", exact: true }).click();
    await expect(page.getByText(/grok 1\.0\.44/)).toBeVisible({ timeout: 170_000 });
    await expect(page.getByRole("button", { name: "Reinstall" })).toBeVisible();
  });

  test("connects a subscription through the displayed device flow", async ({ page }) => {
    test.skip(!process.env.FIRETOWER_E2E_APPROVE_DEVICE, "requires the account owner to approve the displayed code");
    const codeFile = process.env.FIRETOWER_E2E_DEVICE_CODE_FILE;
    if (!codeFile) throw new Error("set FIRETOWER_E2E_DEVICE_CODE_FILE to a private local path");
    test.setTimeout(16 * 60_000);
    await signIn(page);
    await expect(page.getByRole("button", { name: "Reinstall" })).toBeVisible();
    await page.getByRole("button", { name: "Connect an account" }).click();
    await page.getByPlaceholder("Personal Claude, Work, Client Acme…").fill("Grok Playwright E2E");
    await page.getByRole("button", { name: "Continue to sign in" }).click();
    const code = page.getByText(/Enter this code at accounts\.x\.ai/);
    await expect(code).toBeVisible();
    // The runner needs a human to approve the provider's code. The UI keeps
    // polling while this page is open, and a fresh browser can read the row.
    writeFileSync(codeFile, (await code.textContent())?.trim() ?? "", { mode: 0o600 });
    await expect(page.getByText("Account connected")).toBeVisible({ timeout: 15 * 60_000 });
    await page.getByRole("button", { name: "Done" }).click();
    await expect(page.getByText("connected", { exact: true })).toBeVisible();
  });
});
