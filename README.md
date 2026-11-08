# Mistral Vibe ACP validation screenshots

Source: `salim4n/firetower` branch `feat/mistral-vibe-acp`, commit `a47047e2e7cb0dd554610151a65f8dfc1cc6ebdc`.

Captured on 8 November 2026 from the Desktop Vite renderer against the actual
Firetower control plane, isolated Postgres database and a native macOS arm64
worker reached over loopback SSH. No API route, worker or provider response was
mocked for these captures. They are browser-renderer captures, not native Tauri
or mobile runtime acceptance.

- `mistral-installed.png`: the official native `vibe-acp 3.1.0` archive was
  downloaded and checksum-verified by the real worker and detected on its PATH.
  The earlier unapproved test connection shown as failed was left to expire;
  this is not a test of a real account's entitlement.
- `mistral-api-key-form.png`: the real account dialog offers a separate API-key
  mode. No real API key was supplied or validated.
- `mistral-browser-sign-in.png`: the worker used Vibe's delegated ACP auth to
  obtain a real Mistral Console sign-in link. The link was not opened or
  approved. The complete query-bearing URL is not visible in the image.

No Mistral account was available. These screenshots prove installation, UI
wiring and delegated auth start only. Successful authentication, credential
roundtrip, real model turns, tools, controls, skill use/refresh, cancellation,
resume and entitlement acceptance remain unverified with a signed-in account.

Local checks on the source commit: 1,212 Rust tests passed (2 opt-in live tests
ignored by the ordinary suite); strict workspace Clippy, rustfmt and contract
regeneration checks passed; Desktop 649, web 104 and mobile 162 tests passed;
all three clients typechecked. Web and Desktop production builds passed.
The opt-in real Vibe auth-start-and-cancel test was run separately and passed.
