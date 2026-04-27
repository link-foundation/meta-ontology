# 05 — HTTP Microservice (M3)

Goal: serve the ontology over HTTP so other services and the web app can
consume it without bundling the data.

Status: **planned for M3** — not implemented in PR #2.

## Stack

- `axum` (Tokio‑based HTTP framework, minimal, mature in 2026).
- `tokio` 1.x with multi‑threaded runtime.
- `tower-http` for tracing, CORS, compression.
- `serde` + `serde_json` for JSON; `lino-objects-codec` for `text/lino`.
- `tracing` + `tracing-subscriber` for structured logs.

## Endpoints

| Method | Path | Purpose | Notes |
|--------|------|---------|-------|
| `GET` | `/healthz` | Liveness probe | Returns `{"status":"ok"}` |
| `GET` | `/concepts` | List concepts | Paginated; `?limit=`, `?after=` |
| `GET` | `/concepts/:id` | Concept detail | 404 if unknown |
| `GET` | `/concepts/:id/neighbors` | Adjacency | Direction via `?dir=in|out|both` |
| `GET` | `/search` | Substring + label search | `?q=`, `?limit=` |
| `GET` | `/mappings/:id` | Cross‑ontology mappings | |
| `GET` | `/exponents/:id` | Language exponents | `?lang=` filter |

### Content negotiation

- Default `application/json`.
- `Accept: text/lino` returns `lino` text from `lino-objects-codec`.
- `Accept: application/x-lino+json` returns the JSON form of the lino value
  (useful for debugging).

## Configuration

Use `lino-arguments` for CLI flags + `.lenv`:

| Flag | Env | Default | Purpose |
|------|-----|---------|---------|
| `--bind` | `META_ONTOLOGY_BIND` | `127.0.0.1:3000` | Listen address |
| `--data` | `META_ONTOLOGY_DATA` | `data` | Path to data/ |
| `--log-level` | `META_ONTOLOGY_LOG_LEVEL` | `info` | tracing filter |
| `--cors-origin` | `META_ONTOLOGY_CORS_ORIGIN` | `*` | Comma‑separated list |

## Performance

- The ontology is small enough to load once at startup and keep in memory
  behind an `Arc<Ontology>` shared across requests.
- All endpoints are read‑only; no mutex contention.
- Cache `ETag` based on a hash of the data folder so clients can skip
  unchanged responses.

## Security

- Read‑only: no auth required for MVP.
- Rate limit via `tower::limit::ConcurrencyLimitLayer` to prevent trivial
  abuse.
- CORS open by default but configurable.

## Testing

- Spin up a server in a `tokio::test` and hit each endpoint with `reqwest`.
- Add a `cargo run --bin meta-ontology-server` smoke test in CI on PR.

## Deployment (out of scope for PR #2)

- Dockerfile based on `rust:slim` → distroless.
- A sample `docker-compose.yml` for local dev.
- A GitHub Actions workflow that builds and pushes the image to GHCR on
  each main commit.
