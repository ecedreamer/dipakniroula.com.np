# dipakniroula.com.np

Personal website built with **Rust + Axum + PostgreSQL + Askama templates**.

## Stack

- **Rust** (edition 2024) + **Axum 0.8** web framework
- **Diesel ORM** with **PostgreSQL** (async via `diesel-async`)
- **Askama** compile-time templates
- **Google OAuth** for user login; admin auth via email/password + Argon2
- **Gemini API** for quiz generation
- Docker Compose for dev and production deployments

## Prerequisites

- Docker & Docker Compose
- A `.env` file with the following:

```
DATABASE_URL=postgres://user:pass@db:5432/dipak_site
WEB_SUPER_ADMIN=admin@example.com
WEB_PASSWORD=your-admin-password
LOG_DIRECTORY=/tmp/logs/dipakniroula.com.np
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://127.0.0.1:8081/user/auth/google/callback
API_KEY=your-api-key-for-external-access
POSTGRES_USER=user
POSTGRES_PASSWORD=pass
POSTGRES_DB=dipak_site
```

Note: `DATABASE_URL` uses `@db:5432` (Docker service name), not `localhost`.

## Local Development

```bash
# Start the full stack
docker compose up -d

# Follow logs
docker compose logs -f

# Run a one-off command (e.g. seed admin user)
docker compose exec webapp ./dipak_site --fixture
```

- App runs at **http://127.0.0.1:8081**
- PostgreSQL runs on port **5432** (exposed to host)
- Source code is bind-mounted — changes trigger a rebuild via `cargo watch`

### Seed admin user

```bash
# Option A: via the Rust fixture inside the container
docker compose exec webapp cargo run -- --fixture

# Option B: via Python script (from host, with port 5432 exposed)
pip install psycopg2-binary argon2-cffi python-dotenv
python insert_admin.py
```

### Reset the database

```bash
docker compose down -v
docker compose up -d
```

## Production Deployment

The production stack adds **nginx-proxy-manager** as a reverse proxy with automatic SSL.

```bash
# Start the production stack
docker compose -f docker-compose.prod.yml up -d

# Pull latest and rebuild
git pull
docker compose -f docker-compose.prod.yml build
docker compose -f docker-compose.prod.yml up -d
```

### Services

| Service | Container | Purpose |
|---|---|---|
| `webapp` | `dipak_site-webapp-prod` | Rust binary (port 8081) |
| `db` | `postgres_db-prod` | PostgreSQL |
| `nginx-proxy` | `nginx-proxy-prod` | Reverse proxy with SSL (ports 80/443, admin UI on 81) |

### Logs

```bash
docker compose -f docker-compose.prod.yml logs -f webapp
```

Log files are also persisted to the `app_logs` Docker volume at `/tmp/logs/dipakniroula.com.np/`.

### Environment

A `.env` file must exist in the project root. For production, use your actual domain in `GOOGLE_REDIRECT_URI`:

```
GOOGLE_REDIRECT_URI=https://dipakniroula.com.np/user/auth/google/callback
```

## Routes

| Path | Description |
|---|---|
| `/` | Home page |
| `/contact` | Contact form |
| `/blog/list`, `/blog/{id}/detail` | Blog |
| `/my-resume` | Resume / experience |
| `/play-quiz` | Gemini-powered quiz (requires Google OAuth) |
| `/auth/*` | Admin login, panel, CRUD |
| `/user/*` | Google OAuth login, panel |
| `/admin/*` | Quiz management, settings |
| `/api/v1/*` | REST API (requires `x-api-key` header) |
| `/swagger-ui` | API documentation |

## Notes

- Migrations run **automatically** on startup — no manual step needed.
- The `diesel.toml` `migrations_directory` path is stale; `embed_migrations!` in code is the source of truth.
- CORS is locked to `https://dipakniroula.com.np` — change in `main.rs:172` for local testing.
