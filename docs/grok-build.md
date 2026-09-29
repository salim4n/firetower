# Grok Build through ACP

Firetower starts the local Grok Build CLI over `grok agent stdio`. This is the
publisher's ACP interface, separate from Grok Bot and the xAI API. Claude Code,
Codex and Kimi Code keep their existing transports.

## Install and connect

1. In **Configuration → Agents**, choose **Grok Build** and install it on the
   worker that will run the workspace. Firetower currently pins the publisher's
   macOS arm64, Linux arm64 and Linux x86_64 binaries to version **1.0.44** and
   checks their SHA-256 before making them available. Other worker platforms
   report that this version has not yet been verified; a global `grok`
   executable does not count as installed.
2. Add a **Subscription** Grok Build account in Firetower. Choose a worker for
   the device-code flow, open the xAI URL shown by Firetower and approve the
   short code. The login runs in a temporary `GROK_HOME` on that worker.
3. Firetower stores only the resulting `auth.json` in its vault, removes the
   temporary login home, and restores the credential into a private home for
   each workspace. The home is `0700`; credential files are `0600`. The worker
   launches Firetower's pinned binary with `--no-auto-update` and `--no-leader`.
4. Select the Grok Build provider and connected account when starting a
   workspace. The target worker must have the pinned CLI installed.

This connection uses the Grok Build account's device sign-in. Firetower does
not ask for `XAI_API_KEY` or use a host's global Grok login. An API-key account
is separate scope. Billing and entitlement are determined by xAI for the
connected account; Firetower does not infer them from a successful login. An
unentitled or expired account can still fail on a prompt.

## Session behavior

The ACP bridge journals requests and responses for replay after a Firetower
client reconnects. It authenticates with the CLI's `cached_token` method, then
starts or loads a session according to the capability Grok advertises. Unknown
agent notifications remain in the raw journal. Unsupported blocking requests
receive an error; permission requests use Firetower's existing approval path.
The launch explicitly sets `--permission-mode default` (the CLI's ask mode); it does not use
`--always-approve` or session `yoloMode`. Grok may still auto-approve
read-only actions or use provider-side rules, so a tool call alone is not
evidence that Firetower received an approval request.
In the 1.0.44 macOS arm64 probe, a `write` tool changed a file without an
ACP permission request even in default mode. A shell write did request
permission; denial prevented its side effect. Treat the provider's own tool
authorization as distinct from Firetower's approval UI.

Model and reasoning choices come from ACP `configOptions` when present. A
permission-mode picker appears only if the provider advertises one. Firetower
sends the exact option ID back and shows an accepted change only after the
agent confirms it. A nonresponsive option request is reported after 30 seconds.

## Troubleshooting and verification

- **Not installed:** use the Agents screen on the selected worker. A copy on
  the operator's laptop or global worker `PATH` is not Firetower's pinned copy.
- **No connected subscription:** complete the device-code flow in Firetower.
  The worker must reach xAI's authentication endpoints. A login on a laptop is
  not transferred to Firetower automatically.
- **Prompt fails after login:** confirm the account is entitled to use Grok
  Build. Firetower cannot turn an API key into a subscription.
- **Unsupported platform:** macOS arm64 and Linux arm64/x86_64 have verified
  publisher binaries. The Linux binaries answered `--version` and ACP
  `initialize` in isolated containers; only macOS arm64 has an authenticated
  ACP probe so far. Other platforms fail closed.

The feasibility probe on macOS arm64 completed device-code login, copied only
`auth.json` into a fresh empty home, authenticated ACP, opened a session and
finished a streamed text prompt. This is a CLI proof, not acceptance of the
Firetower control-plane journey. Before release, exercise installation and
connection through Firetower, two prompts, approvals and denial with observed
effects, cancellation, restart, reconnect, the native mobile app and the
desktop app on the exact final build. Record fixture and real-provider results
separately in the pull request.

Publisher references: [installation](https://docs.x.ai/build/overview),
[device sign-in](https://docs.x.ai/build/cli/reference), and
[ACP](https://docs.x.ai/build/cli/headless-scripting).
