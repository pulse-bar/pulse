# OAuth & credentials

Pulse uses **OAuth 2.0 with PKCE** as the preferred auth method for SaaS
integrations (Atlassian, GitHub, Linear, Slack, Notion …) and falls back
to Personal Access Tokens for self-hosted, on-prem, or air-gapped
environments. Every credential — OAuth tokens or PATs — is stored in the
**OS keychain**, never in plaintext config.

## The single auth surface

The `crates/auth` crate defines one trait every integration reads from:

```rust
#[async_trait]
pub trait CredentialStore: Send + Sync {
    async fn read(&self, key: &str)   -> AuthResult<Credential>;
    async fn store(&self, key: &str, c: &Credential) -> AuthResult<()>;
    async fn delete(&self, key: &str) -> AuthResult<()>;
}
```

`Credential` is a typed enum:

```rust
pub enum Credential {
    Bearer { token: String },
    Basic  { token: String },
    OAuth2 { tokens: TokenSet },
}
```

`TokenSet` carries `access_token`, `refresh_token`, `expires_at`, and
knows how to render a correct `Authorization` header. Refresh is handled
by `OAuthClient::refresh()` against the provider's token URL.

## Why this shape

- **One trait, every integration uses it.** Jira reads tokens through
  `CredentialStore`; the upcoming GitHub Issues, Linear, Slack, and
  Notion enrichers all read through the same trait. No integration ever
  hits the keychain or HTTP directly.
- **Storage backend is swappable.** The default is
  `KeychainCredentialStore` (Keychain on macOS, Secret Service on Linux,
  Credential Manager on Windows). For headless dev containers a
  `FileCredentialStore` with `0600` perms can be added in 30 lines —
  no integration code changes.
- **PKCE solves the "no client secret in a desktop binary" problem.**
  `OAuthClient` generates a per-flow code-verifier, hashes it as the
  challenge, and only exchanges the code with the matching verifier
  presented at the token endpoint. The shipped `client_id` is public;
  that's the whole point of PKCE.

## Built-in OAuth providers

| Provider     | Authorize URL                                  | Default scopes                                                  |
| ------------ | ---------------------------------------------- | --------------------------------------------------------------- |
| `atlassian`  | `https://auth.atlassian.com/authorize`         | `read:jira-user read:jira-work offline_access`                  |
| `github`     | `https://github.com/login/oauth/authorize`     | `repo read:user`                                                |
| `custom`     | (caller supplies)                              | (caller supplies)                                               |

Add a provider by adding a file under `crates/auth/src/providers/`,
exposing a `LazyLock<OAuthProviderConfig>`, and listing it in
`OAuthProviderId` + `for_id()`.

## Redirect URI

`pulse://oauth/callback`

The Tauri shell will receive this via `tauri-plugin-deep-link` (wiring
pending — see follow-up below). Browser → OS deep-link handler →
Pulse's command dispatch → `oauth_complete` Tauri command → tokens
land in the keychain via `CredentialStore`.

## Flow

```
[ React UI ]    user clicks "Connect to Atlassian"
     │
     ▼
oauth_begin( provider, site_id, client_id, scopes )
     │
     ▼  PkceFlow created, stored in OAuthClient.flows[state]
[ Pulse ] returns authorize_url
     │
     ▼  open in user's browser (system browser, not WebView)
[ Atlassian ] user authorises
     │
     ▼  redirect: pulse://oauth/callback?code=…&state=…
[ OS ] deep-link → Pulse
     │
     ▼
oauth_complete( provider, state, code )
     │
     ▼  PkceFlow looked up by `state`, code+verifier exchanged for tokens
[ Pulse ] tokens stored in keychain via CredentialStore
     │
     ▼  emits pulse://oauth-completed
[ React UI ] refreshes site status — "✓ tokens stored"
```

## Current status

Foundation **shipped** — `crates/auth` with PKCE, the `Credential`
enum, the `CredentialStore` trait, the keychain backend, the OAuth
client, and the `oauth_begin` / `oauth_complete` Tauri commands.
Atlassian and GitHub provider configs are baked in.

Pending:

- [ ] Register Pulse as an OAuth app at Atlassian → embed the public
  `client_id` and ship.
- [ ] Register a GitHub OAuth app (or App, ideally) → same.
- [ ] Wire `tauri-plugin-deep-link` so `pulse://oauth/callback` is
  received and routed to `oauth_complete`. Per-platform setup:
  - macOS: `Info.plist` `CFBundleURLTypes`.
  - Linux: `.desktop` file with `MimeType=x-scheme-handler/pulse;`.
  - Windows: registry entry in installer.
- [ ] Add automatic token refresh — call `OAuthClient::refresh()` when
  `TokenSet::is_expired()` returns true; persist refreshed tokens.
- [ ] Add GitHub App support (different from OAuth: installation token
  exchange via JWT) once needed for fine-grained per-org perms.

Until OAuth is fully wired, users can configure each Jira site with
**Bearer (PAT)** or **Basic (email + API token)** — those use the same
`CredentialStore` and work today.

## Adding an integration

When you add a new integration (e.g. `extensions/enrichment-linear`):

1. Define the per-integration site/workspace config in
   `crates/core/src/model.rs`.
2. Build the enricher around `Arc<dyn CredentialStore>` — never read the
   keychain directly.
3. Use `site_credential_key(site_id)` (or the equivalent) so credentials
   are namespaced per integration: `linear:<workspace_id>` ≠ `jira:<site_id>`.
4. For OAuth: add the provider config to `crates/auth/src/providers/`.
5. UI: reuse `JiraSitesEditor` as the template — it already handles
   "Bearer / Basic / OAuth 2.0" auth-kind selection.

The integration code never knows whether the credential came from a PAT,
an OAuth flow, the Mac keychain, or a future cloud secret store.
