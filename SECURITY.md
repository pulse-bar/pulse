# Security policy

## Reporting a vulnerability

Email security reports to **security@iamdotk.dev** (or open a private
[GitHub Security Advisory](https://github.com/pulse-bar/pulse/security/advisories/new)).
Please don't file public issues for vulnerabilities until we've had a
chance to ship a fix.

We aim to:

- Acknowledge receipt within **2 working days**.
- Triage and confirm impact within **5 working days**.
- Ship a patched release within **30 days** of confirmation, or sooner
  for high-severity issues.

## Scope

Pulse is local-first. A useful report typically falls into one of:

- **Path traversal / arbitrary file read** in the watcher or storage
  layer that exposes data outside `~/.claude/projects` and the app
  data directory.
- **SQL injection** through user-controlled fields (Settings → Jira
  config, branch regex).
- **Privilege escalation** via the Tauri shell, plugins, or autostart.
- **Credential leakage** in the SQLite store, logs, or crash reports.
  Pulse currently stores no credentials, so any path that *would*
  store one is itself a vulnerability.
- **Malicious-transcript handling.** The parser must treat every
  JSONL line as untrusted input; crashes are bugs, RCE is a security
  bug.

Out of scope:

- Issues that require an attacker to already have arbitrary code
  execution as your user account.
- Tauri framework or upstream dependency vulnerabilities — please
  report those to the corresponding project. We will pull in patched
  versions promptly.

## Hardening notes

- Tauri capabilities are minimised in
  `apps/desktop/shell/capabilities/default.json`.
- `macOSPrivateApi` is enabled for popover transparency and prevents
  App Store distribution; this is a deliberate trade-off.
- The watcher is read-only on `~/.claude/projects`. The only paths it
  writes are the SQLite database and Pulse's own settings.

## Disclosure

Once a fix has shipped, we publish a CVE-style advisory in
GitHub Security Advisories that credits the reporter (unless you ask
us not to).
