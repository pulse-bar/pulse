# Connecting Pulse to Jira

Pulse uses **OAuth 2.0 with PKCE** to connect to Atlassian — the same
secure flow you'd see clicking *"Sign in with Google"* on the web.
You'll click one button in Pulse, authorize in your browser, and you're
done. No API tokens to copy, no plaintext credentials.

The one-time setup below takes about **3 minutes** and is performed by a
single Atlassian admin per workspace. After that, every member of your
team simply pastes the resulting **client ID** into Pulse.

---

## One-time admin setup

### 1. Create an Atlassian OAuth 2.0 (3LO) app

1. Sign in at https://developer.atlassian.com/console/myapps/ as an
   Atlassian-site admin.
2. Click **Create** → **OAuth 2.0 integration**.
3. Name it whatever you like (e.g. *"Pulse"* or *"Pulse — Engineering"*).
4. Click **Create**.

### 2. Configure permissions

On the **Permissions** tab, click **Add** next to **Jira API** and
enable these scopes:

- `read:jira-user`
- `read:jira-work`
- `offline_access` *(needed for token refresh — Pulse re-authenticates silently every few hours)*

### 3. Configure the Authorization callback

On the **Authorization** tab → **OAuth 2.0 (3LO)** → **Configure**:

Add these three callback URLs (Pulse picks whichever local port is free
at runtime):

```
http://127.0.0.1:19834/callback
http://127.0.0.1:19835/callback
http://127.0.0.1:19836/callback
```

Click **Save changes**.

### 4. Copy the client ID

On the **Settings** tab, copy the **Client ID** (it looks like a long
hex string, e.g. `aBcD1234EfGh5678IjKl9012`).

> The **client secret** is **not** needed. Pulse uses PKCE, which is the
> standard secret-less flow for desktop apps.

### 5. Distribute the client ID

Share the client ID with your team — paste it in Slack, your wiki,
your team's onboarding doc. It's not a secret; it just identifies the
OAuth app. Token security is handled by PKCE on each user's machine.

---

## Per-user connection

Each user does this once, for each Jira site they want to track:

1. Open Pulse → **Settings** → **Integrations** → **Jira** → **Add Jira site**.
2. Fill in:
   - **Site label** — anything memorable, e.g. *"Acme Engineering"*.
   - **Base URL** — `https://yourcompany.atlassian.net`.
   - **Project keys** — comma-separated, e.g. `PROJ, WEB, INFRA`.
   - **Authentication** — leave on the default **OAuth 2.0**.
   - **OAuth client ID** — paste the client ID your admin gave you.
3. Click **Connect with Atlassian**.
4. Your default browser opens at `auth.atlassian.com`. Sign in if
   prompted, click **Accept**.
5. Browser shows *"Pulse is connected. You can close this tab."*
6. Pulse window updates to **✓ Connected**.

That's it. Tokens land in the **OS keychain** — never in plaintext
config, never in shell history. Pulse refreshes them silently before
they expire, so you don't have to reconnect.

---

## Self-hosted Jira / Jira Data Center

OAuth 2.0 (3LO) is **Cloud-only**. For self-hosted Jira (Data Center
or Server), use one of:

- **Bearer (PAT)** — *Settings → Integrations → Jira → Add Jira site →
  Authentication: Bearer*. Generate a Personal Access Token in your
  Jira profile → **Personal access tokens** → **Create token**. Paste
  it into Pulse — stored in keychain.
- **Basic (email + API token)** — same flow with email + token.

Both work today and use the same `CredentialStore` keychain backend as
the OAuth flow.

---

## Troubleshooting

**"OAuth client ID is required"** — you haven't pasted the ID yet. Get
it from your admin (step 5 above) or register an app yourself
(steps 1–4).

**Browser opens but redirect fails / "site can't be reached"** — the
callback URL isn't registered with the Atlassian app. Re-do **step 3**
above and make sure all three `http://127.0.0.1:198xx/callback` URLs
are listed.

**"OAuth callback timed out"** — you have 3 minutes from clicking
**Connect** to finish the browser dance. Just click **Connect with
Atlassian** again.

**`status 401: invalid_grant`** — the OAuth app's permissions don't
include the needed scopes. Re-do **step 2**, save, and reconnect from
Pulse.

**Pulse Settings shows "auth: keychain: get: No matching entry…"** —
the user clicked **Test connection** before completing the connect
flow. Click **Connect with Atlassian** first, then test.

---

## Why a client ID per workspace, not built into Pulse?

Two reasons:

1. **Security boundary.** The OAuth app's installed scopes, audit logs,
   and revocation are owned by **your** Atlassian admin — not us. Your
   compliance and SOC 2 team can audit a Pulse-shaped OAuth app the same
   way they audit any Atlassian Connect app.
2. **No vendor lock-in.** Pulse is local-first. Tokens issued for your
   OAuth app go to your users, not to a Pulse-controlled cloud service.
   Even if Pulse the project disappears tomorrow, your tokens keep
   working until you choose to revoke them.

Down the road we may also publish a public Pulse OAuth app for users
who don't want to register their own — but the BYO pattern stays as
the recommended path for any organisation with security policies worth
calling that.
