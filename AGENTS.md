# dipakniroula.com.np — Agent Guide

## Stack
- **Rust** (edition 2024) + **Axum 0.8** web framework
- **Diesel ORM** with **PostgreSQL** (async via `diesel-async`, sync for startup migrations)
- **Askama** compile-time templates (`templates/`)
- Custom DB-backed sessions (no external session crate)
- **Google OAuth** for user login; admin auth via email/password + Argon2
- **Gemini API** (`gemini-rust` crate) for quiz generation
- **utoipa** + **Swagger UI** at `/swagger-ui`
- Containerized (Docker Compose for dev + prod)

## Database
- PostgreSQL (NOT SQLite — README is stale)
- Migrations run **automatically at startup** via `diesel_migrations::embed_migrations!` in `main.rs:161`
- To regenerate schema: `diesel print-schema` (but `diesel.toml` has a stale hardcoded path — set `DATABASE_URL` env var instead)
- `src/schema.rs` is manually checked in (auto-generated)

## Required env vars (`.env`)
```
DATABASE_URL, WEB_SUPER_ADMIN, WEB_PASSWORD, LOG_DIRECTORY,
GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_REDIRECT_URI,
API_KEY
```

## Dev commands
| Action | Command |
|---|---|
| Run dev server (port 8081) | `cargo watch --ignore dipakdb.* -x run` or `cargo run` |
| Run release build | `cargo build --release` |
| Seed admin user | `cargo run -- --fixture` (also available: `python insert_admin.py`) |
| Tests | `cargo test` (no tests exist yet) |

## Architecture
- **Entrypoint**: `src/main.rs` — builds router, runs migrations, starts on `0.0.0.0:8081`
- **Router structure** (all relative to `/`):
  - `/` — home page
  - `/contact` — contact form (GET/POST, CSRF-protected, honeypot field)
  - `/auth/*` — admin auth routes (login/logout/panel/CRUD social links & messages)
  - `/user/*` — OAuth user routes (Google login/panel/logout)
  - `/admin/*` — admin-only section (quiz management)
  - `/play-quiz` — quiz feature (requires OAuth session)
  - `/api/v1/*` — REST API (requires `x-api-key` header)
  - `/swagger-ui` — Swagger docs
  - `/static/*`, `/media/*` — static file serving
- **AppState**: holds `db_pool: DbPool` + `csrf_config: CsrfConfig`
- **Middleware stack**: `optional_session_middleware` (public) → route-specific `session_middleware` (admin) / `user_session_middleware` (oauth) → `security_headers_middleware` (CSP, HSTS, etc.) → CORS (origin-locked to `https://dipakniroula.com.np`)
- **Body limit**: 20 MB (`DefaultBodyLimit::max(20 * 1024 * 1024)`)

## Module layout
| Directory | Purpose |
|---|---|
| `src/auth/` | Admin auth (login, logout, dashboard, social link CRUD, message management) |
| `src/oauth/` | Google OAuth flow, user panel |
| `src/blog/` | Blog CRUD + repository pattern |
| `src/resume/` | Resume/experience CRUD |
| `src/quiz/` | Gemini-powered quiz generation, session management, admin quiz views |
| `src/api/` | REST API v1 (blogs, experiences, messages) + `x-api-key` auth |
| `src/message/` | Contact message repository |
| `src/utils/` | `AppError` enum, crypto helpers |

## Notable quirks
- `diesel.toml` migrations dir is a stale hardcoded path — `embed_migrations!` in code is the source of truth
- CORS origin is hardcoded to `https://dipakniroula.com.np` — update in `main.rs:172` for local dev
- `security_headers_middleware` injects CSP, HSTS, etc. on every response
- Session flash messages are stored in the `sessions.data` column as JSON
- Quiz sessions stored in `quiz_sessions` table with UUID key; cleaned up after submission
- The `website_url` honeypot field on the contact form is invisible — bots that fill it get silently redirected
- `/media/` and `/media/summernote/` are gitignored; created at runtime for uploads
