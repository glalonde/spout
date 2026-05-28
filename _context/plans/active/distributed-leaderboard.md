# Distributed Leaderboard Design Proposal

Date: 2026-05-10  
Status: Proposal  
Scope: Standalone backend design only; no game integration yet.

## Goal

Add a simple global leaderboard that can eventually receive scores from the
web, iOS, Android, native macOS, and store-distributed builds such as Steam.
The first implementation should be separate from the game code so it can be
developed, tested, deployed, and iterated independently before being wired into
Spout.

The leaderboard does not need to be cheat-proof. It should make arbitrary score
submission slightly nontrivial by requiring a server-created run, a short-lived
single-use submit token, basic plausibility checks, and rate limits.

## Non-Goals

- No platform-specific leaderboard dependency in the first version. Steam,
  Game Center, and Google Play Games can be mirrors or filters later.
- No user accounts in the first version.
- No server-authoritative simulation.
- No deterministic replay validation in the first version.
- No game-code changes until the standalone service is usable via curl.
- No dependency on Spout internals from the backend. The service knows only
  about runs, scores, rulesets, platforms, and lightweight proof fields.

## Proposed Repo Layout

Create a standalone service under:

```text
services/leaderboard/
  package.json
  wrangler.jsonc
  README.md
  .dev.vars.example
  migrations/
    0001_initial.sql
  src/
    index.ts
    tokens.ts
    validation.ts
```

Keep the game integration for a later phase:

```text
src/leaderboard.rs
```

The service should be fully testable before `src/leaderboard.rs` exists.

## Hosting Choice

Use Cloudflare Workers plus D1 for the first version.

Reasons:

- HTTPS endpoint with no server or VM to maintain.
- D1 is enough for the data shape: short run rows plus score rows.
- Wrangler config, migrations, and Worker code can live in the repo.
- Local development can use `wrangler dev` and local D1.
- The service can later be moved to another host because the game sees only
  HTTPS JSON endpoints.

## Service API

All endpoints return JSON.

### `GET /health`

Purpose: deployment and smoke-test check.

Response:

```json
{
  "ok": true,
  "service": "spout-leaderboard",
  "version": "0.1.0"
}
```

### `POST /v1/runs/start`

Purpose: create a score-eligible run before gameplay begins.

Request:

```json
{
  "player_id": "anon_7f52a6f0",
  "display_name": "GEOFF",
  "os": "web",
  "distribution": "web",
  "app_version": "0.1.3",
  "ruleset_id": "classic-2026-05"
}
```

Validation:

- `player_id`: optional. If present, 3-80 characters, stable anonymous ID
  persisted by the client. If absent, the server allocates one and returns it
  in the response; the client must persist whatever the server returns. The
  field is a display/grouping hint, never an auth token: anyone can claim any
  `player_id`, and no feature should rely on its authenticity.
- `display_name`: optional, 1-16 printable ASCII characters after trimming.
- `os`: one of `web`, `ios`, `android`, `macos`, `windows`, `linux`,
  `unknown`. Identifies the operating system the client is running on.
- `distribution`: one of `web`, `appstore`, `playstore`, `steam`, `direct`,
  `unknown`. Identifies the distribution channel. `os` and `distribution`
  are stored independently so that, e.g., Steam-on-macOS is unambiguous.
- `app_version`: required semantic-ish version string, max 32 characters.
- `ruleset_id`: required, max 64 characters.

Response:

```json
{
  "run_id": "run_01JY0000000000000000000000",
  "player_id": "anon_7f52a6f0",
  "seed": 123456789,
  "nonce": "base64url-random-run-nonce",
  "ruleset_id": "classic-2026-05",
  "submit_token": "base64url-signed-token",
  "expires_at": "2026-05-11T12:30:00Z"
}
```

Notes:

- The server chooses the run seed.
- `player_id` is echoed back. If the client supplied one it is returned
  unchanged; if not, the server allocates and returns one. The client must
  persist the returned value.
- `nonce` is the HMAC key the client uses to compute `proof_hash` at submit
  time. Because the nonce is server-issued and never leaves the run row, a
  third party who didn't call `/runs/start` cannot produce a valid hash for
  someone else's run. See "Token Design" below.
- `submit_token` expires after 30 minutes. The token is signed and is the
  authoritative source for its own expiry; the `runs.token_expires_at` column
  exists only to support cleanup queries.
- A run can be submitted at most once.
- If the game later supports local-only practice runs, those should not call
  this endpoint.

### `POST /v1/runs/submit`

Purpose: submit the final score for a previously started run.

Request:

```json
{
  "run_id": "run_01JY0000000000000000000000",
  "submit_token": "base64url-signed-token",
  "player_id": "anon_7f52a6f0",
  "display_name": "GEOFF",
  "score": 18420,
  "duration_ms": 91344,
  "os": "web",
  "distribution": "web",
  "app_version": "0.1.3",
  "ruleset_id": "classic-2026-05",
  "seed": 123456789,
  "proof_hash": "base64url-hmac-sha256",
  "stats": {
    "progress_height": 15720,
    "time_bonus_score": 2700,
    "max_speed": 81.4,
    "frames": 5480
  }
}
```

`proof_hash` is computed by the client as:

```text
proof_hash = base64url(HMAC-SHA256(
  key   = nonce,
  input = canonical_json({ run_id, score, duration_ms, seed, stats })
))
```

`canonical_json` means the JSON object serialized with sorted keys and no
insignificant whitespace. The server reproduces the same hash from the row's
`nonce` and the submitted fields and rejects mismatches. This binds the
server-issued nonce into the submission and makes a forged submit require
either a real `/runs/start` call or knowledge of someone else's nonce.

Validation:

- Token signature is valid.
- Token has not expired.
- Token references the same `run_id`, `player_id`, `seed`, and `ruleset_id`.
- The run exists.
- The run has not already been submitted (enforced atomically — see below).
- `proof_hash` recomputes correctly using the row's `nonce`.
- `score >= 0`.
- `duration_ms` is within an allowed range, initially 1 second to 30 minutes.
- `score` and `stats` pass basic plausibility checks.
- `ruleset_id` is accepted by server config.

The atomic "not yet submitted" check must use a single SQL statement so
concurrent retries can't both pass:

```sql
UPDATE runs
SET submitted_at = ?1
WHERE id = ?2 AND submitted_at IS NULL;
```

If `changes()` is zero the run was already submitted; treat as the
idempotent-replay case below. Otherwise insert into `scores`. Wrap both in
`BEGIN IMMEDIATE ... COMMIT` so the score insert and the run update either
both land or neither does.

Response:

```json
{
  "accepted": true,
  "score_id": "score_01JY0000000000000000000000",
  "rank": 42,
  "idempotent_replay": false
}
```

Idempotent retry: if a submission arrives for an already-submitted run and
the payload's `score`, `duration_ms`, `seed`, and `proof_hash` match the
stored score, return the same response with `idempotent_replay: true` and
the existing `score_id` and `rank`. This makes flaky-network retries safe.

If the replayed payload differs from the stored score, reject:

```json
{
  "accepted": false,
  "error": "run_already_submitted"
}
```

`rank` is computed against the global view of the ruleset using the
`scores_global_idx` covering `(ruleset_id, score DESC, duration_ms ASC,
created_at ASC)`:

```sql
SELECT 1 + COUNT(*) FROM scores
WHERE ruleset_id = ?1
  AND ( score > ?2
     OR (score = ?2 AND duration_ms < ?3)
     OR (score = ?2 AND duration_ms = ?3 AND created_at < ?4) );
```

Do not replace this with `ORDER BY ... LIMIT N` — the index makes the
COUNT cheap and the limit query loses the tie-break ordering.

### `GET /v1/leaderboard`

Purpose: fetch top scores.

Query parameters:

- `ruleset_id`: required.
- `os`: optional. If omitted, return scores across all operating systems.
- `distribution`: optional. If omitted, return scores across all distribution
  channels.
- `limit`: optional, default 20, max 100.
- `player_id`: optional. If present and the player has a score outside the
  top window, the response includes a `player_score` field with that score
  and its rank. The field is omitted otherwise.

Example:

```text
GET /v1/leaderboard?ruleset_id=classic-2026-05&os=ios&limit=20
```

Response:

```json
{
  "ruleset_id": "classic-2026-05",
  "os": "ios",
  "distribution": null,
  "scores": [
    {
      "rank": 1,
      "score": 28400,
      "display_name": "GEOFF",
      "os": "ios",
      "distribution": "appstore",
      "app_version": "0.1.3",
      "created_at": "2026-05-11T12:00:00Z"
    }
  ]
}
```

## D1 Schema

Initial migration:

```sql
CREATE TABLE runs (
  id TEXT PRIMARY KEY,
  player_id TEXT NOT NULL,
  display_name TEXT,
  os TEXT NOT NULL,
  distribution TEXT NOT NULL,
  app_version TEXT NOT NULL,
  ruleset_id TEXT NOT NULL,
  seed INTEGER NOT NULL,
  nonce TEXT NOT NULL,
  token_expires_at INTEGER NOT NULL,
  submitted_at INTEGER,
  created_at INTEGER NOT NULL
);

CREATE TABLE scores (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL UNIQUE,
  player_id TEXT NOT NULL,
  display_name TEXT,
  score INTEGER NOT NULL,
  duration_ms INTEGER NOT NULL,
  os TEXT NOT NULL,
  distribution TEXT NOT NULL,
  app_version TEXT NOT NULL,
  ruleset_id TEXT NOT NULL,
  seed INTEGER NOT NULL,
  proof_hash TEXT NOT NULL,
  stats_json TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(id)
);

CREATE INDEX scores_global_idx
  ON scores (ruleset_id, score DESC, duration_ms ASC, created_at ASC);

CREATE INDEX scores_os_idx
  ON scores (ruleset_id, os, score DESC, duration_ms ASC, created_at ASC);

CREATE INDEX scores_distribution_idx
  ON scores (ruleset_id, distribution, score DESC, duration_ms ASC, created_at ASC);

CREATE INDEX scores_player_idx
  ON scores (ruleset_id, player_id, score DESC);

CREATE INDEX runs_token_expires_idx
  ON runs (token_expires_at) WHERE submitted_at IS NULL;

CREATE INDEX runs_player_created_idx
  ON runs (player_id, created_at DESC);
```

Ranking tie-breakers:

1. Higher score wins.
2. Shorter duration wins.
3. Earlier submission wins.

`scores.created_at` is set when the row is inserted, not when the run was
played. A player who finishes quickly but submits a few seconds later than
a tied player can lose the tie-break. This is intentional and acceptable
for an arcade game; deterministic-replay-time would require trusting
`duration_ms` further than we want.

`scores.stats_json` shape is fixed per `ruleset_id`. Treat it as part of the
ruleset contract: any change to its keys is a new `ruleset_id`. This lets
future anti-abuse checks query specific stats fields without a migration
penalty.

## Token Design

Do not embed a private API secret in the game.

Use a server-signed submit token. The Worker stores the signing secret in
Cloudflare secrets as `SUBMIT_TOKEN_SECRET`.

Payload:

```json
{
  "run_id": "run_...",
  "player_id": "anon_...",
  "seed": 123456789,
  "ruleset_id": "classic-2026-05",
  "expires_at": 1778502600
}
```

Encoding:

```text
base64url(json_payload) + "." + base64url(hmac_sha256(payload, secret))
```

Worker validation uses Web Crypto HMAC APIs. Local tests can use the same path
under Miniflare/Wrangler.

## Anti-Abuse Checks

First version:

- Require `POST /v1/runs/start` before score submission.
- Require a valid short-lived signed token.
- Mark each run as submitted atomically with the score insert (single
  `UPDATE ... WHERE submitted_at IS NULL` plus `INSERT` inside one
  `BEGIN IMMEDIATE` transaction). See the submit endpoint section.
- Reject duplicate submissions for the same run, except for the
  identical-payload idempotent-replay case.
- Require `proof_hash` to recompute correctly using the row's `nonce`.
- Reject unknown `ruleset_id` once server config contains an allowlist.
- Reject impossible duration and score ranges.
- Reject display names with control characters or excessive length.
- Return CORS only for known web origins once the web build has a stable URL.

Operational hygiene:

- Run a scheduled Worker (cron trigger) once per day to delete un-submitted
  runs whose tokens have expired:
  `DELETE FROM runs WHERE submitted_at IS NULL AND token_expires_at < ?1`.
  The partial index `runs_token_expires_idx` keeps this cheap.

Future version:

- Add per-IP and per-player rate limiting with Cloudflare rate limiting or a
  lightweight D1-backed window.
- Add rolling proof hashes at periodic checkpoints.
- Store compressed input/checkpoint traces for top scores.
- Add a manual moderation field such as `visibility = 'public' | 'hidden'`.
- Add an `app_version` allowlist if a regression in a shipped client makes
  filtering necessary. Not needed up front: `ruleset_id` already pins the
  scoring contract.

## CORS

Native/mobile builds are not constrained by browser CORS. Web builds are.

Initial local development:

- Allow `http://localhost:*` for dev.
- Allow the deployed Spout web origin once known.

Production:

- Do not use `Access-Control-Allow-Origin: *` for score submission.
- `GET /v1/leaderboard` may be public and cacheable.
- `POST /v1/runs/start` and `POST /v1/runs/submit` should allow only known web
  origins, while still accepting native/mobile requests that do not send a web
  origin.

CORS is a browser-only mechanism. Any non-browser HTTP client (curl, a native
app, a malicious script) bypasses it entirely. CORS here exists to keep
arbitrary third-party web pages from submitting scores on behalf of a logged-
in browser user; it is not an authorization boundary. The token, atomic-submit
check, and `proof_hash` are what gate submission.

## Caching

`GET /v1/leaderboard` can be cached briefly:

```text
Cache-Control: public, max-age=15
```

Cloudflare's edge cache keys on full request URL by default, so distinct
combinations of `ruleset_id`, `os`, `distribution`, `limit`, and `player_id`
get distinct cache entries automatically. Do not add a `Vary` header that
would defeat this. If `player_id` becomes a common query, consider using the
Cache API explicitly to drop it from the cache key for the public top-N
portion.

Do not cache run creation or score submission responses.

## Local Development

Expected commands from `services/leaderboard/`:

```bash
npm install
npx wrangler d1 migrations apply spout-leaderboard --local
npx wrangler dev
```

Smoke tests:

```bash
curl http://localhost:8787/health
curl -X POST http://localhost:8787/v1/runs/start \
  -H 'content-type: application/json' \
  -d '{"player_id":"anon_local","display_name":"GEOFF","os":"web","distribution":"web","app_version":"0.1.3","ruleset_id":"classic-2026-05"}'
```

The README should include a tiny script or documented curl flow that starts a
run, extracts the token, submits a score, and fetches the leaderboard.

## Cloudflare Setup

One-time account setup:

```bash
npx wrangler login
npx wrangler d1 create spout-leaderboard
npx wrangler secret put SUBMIT_TOKEN_SECRET
```

After `d1 create`, paste the returned `database_id` into `wrangler.jsonc`.

Remote migration:

```bash
npx wrangler d1 migrations apply spout-leaderboard --remote
```

Deploy:

```bash
npx wrangler deploy
```

The first deploy can use the default Workers URL. A custom domain such as
`leaderboard.spoutgame.com` can be added later without changing the protocol.

## Wrangler Config Shape

```jsonc
{
  "name": "spout-leaderboard",
  "main": "src/index.ts",
  "compatibility_date": "2026-05-10",
  "d1_databases": [
    {
      "binding": "DB",
      "database_name": "spout-leaderboard",
      "database_id": "fill-after-d1-create"
    }
  ]
}
```

Local development should use `.dev.vars` for non-committed values:

```text
SUBMIT_TOKEN_SECRET=local-development-secret
```

Commit `.dev.vars.example`, not `.dev.vars`.

## Later Game Integration Boundary

When the service is ready, add a small Rust client module. The rest of Spout
should depend on a thin client interface rather than on an HTTP crate
directly, but the exact trait shape (sync vs. async, channel vs. event
queue, how it integrates with the wgpu frame loop) should be designed when
the integration lands, not now. Two implementations to plan for:

- A no-op client used by default until the feature is wired in.
- An HTTP client that works on both native (reqwest or ureq) and WASM
  (browser fetch).

Hard constraint: the game must never block a frame on networking. Whatever
shape the client takes, requests must complete asynchronously and surface
results through something the main loop polls between frames.

Network touch points:

- Title screen: fetch leaderboard if configured.
- Run start: request `run_id`, `seed`, `nonce`, and `submit_token`. The
  client must keep the `nonce` for the duration of the run to compute
  `proof_hash` at submit time.
- Game over: submit score with the `proof_hash`. On a network error, retry
  the same payload — duplicate retries are idempotent.
- Game over/title: refresh leaderboard.

Config:

```toml
leaderboard_enabled = false
leaderboard_base_url = "https://spout-leaderboard.example.workers.dev"
leaderboard_ruleset_id = "classic-2026-05"
```

Platform notes (`os` axis, except where noted):

- WASM (`os: "web"`): use browser fetch; server must return CORS headers.
- iOS (`os: "ios"`): use HTTPS; avoid plain HTTP because App Transport
  Security dislikes it.
- Android (`os: "android"`): add internet permission when the Android target
  exists.
- macOS / Windows / Linux native: normal HTTPS.

The `distribution` axis (`web`, `appstore`, `playstore`, `steam`, `direct`)
is independent of `os` and is set by the build, not detected at runtime.

## Ruleset Policy

Every scoring-affecting gameplay change should create a new `ruleset_id`.

Examples:

- `classic-2026-05`
- `daily-2026-05-11`
- `classic-0.2.0`

The leaderboard should group by `ruleset_id`. Platform-specific views are
filters within a ruleset, not separate score systems.

## Implementation Milestones

1. Commit this design proposal.
2. Scaffold `services/leaderboard/` with Worker, D1 migration, README, and local
   dev config example.
3. Implement `GET /health`.
4. Implement D1 schema and local migration.
5. Implement submit token signing and validation primitives, plus
   `POST /v1/runs/start` (which issues the first real tokens).
6. Implement `POST /v1/runs/submit` with the atomic submit transaction,
   `proof_hash` verification, and idempotent-replay handling.
7. Implement `GET /v1/leaderboard`.
8. Add the cron-triggered cleanup of expired un-submitted runs.
9. Add curl smoke-test documentation.
10. Deploy to a Cloudflare Workers dev URL.
11. Add a no-op leaderboard client to Spout.
12. Add the HTTP client behind a disabled-by-default config toggle.
13. Wire start-run and submit-score only after the service is independently
    working.
14. Add UI for player display name and leaderboard display.

## Open Questions

- Should global leaderboards require a server-provided seed from day one, or
  allow client-generated seeds for the classic endless mode?
- What is the first stable `ruleset_id` once scoring is ready?
- Should score submission be opt-in, or enabled by default with anonymous
  player IDs?
- Display name is currently restricted to printable ASCII, which excludes
  most non-Latin players. The pixel arcade font may justify ASCII-only;
  decide explicitly before shipping the name UI.
- Is `duration_ms` a tie-breaker for all modes, or only time-attack/daily
  challenge modes?
- Should the first web origin be a Cloudflare Pages site, GitHub Pages, or
  another host?
- Do we want a `GET /v1/runs/{id}` debug endpoint (gated behind a header or
  removed before public deploy), or is D1 console access enough for the
  same job?
- JWT (HS256) vs. the bespoke `base64url(payload).base64url(sig)` token
  format: same security properties, but JWT comes with broader tooling and
  recognizable shape. Pick before implementing token signing.

## Completion Criteria For Backend MVP

- Local `wrangler dev` works.
- Local D1 migration applies.
- Curl can start a run, submit a score, and fetch it from the leaderboard.
- Duplicate run submission is rejected.
- Expired or tampered submit tokens are rejected.
- Leaderboard can be filtered by `ruleset_id`, `os`, and `distribution`.
- No Spout gameplay/rendering code is touched by the backend MVP.
