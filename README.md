# Timey

Arbeitszeiterfassung: Rust/Axum-API, SvelteKit-Frontend, SQLite, Docker Compose hinter Traefik.

## Entwicklung

```bash
# API
cd api
DATABASE_URL=sqlite://data/timey.db \
BOOTSTRAP_ADMIN_USERNAME=admin \
BOOTSTRAP_ADMIN_PASSWORD=changeme \
cargo run

# Frontend
cd web
npm install
npm run dev
```

Frontend: http://127.0.0.1:5173 (Vite proxied `/api` → `:3000`).

Login: Bootstrap-Admin aus den Env-Variablen.

## Tests

```bash
cd api && cargo test && cargo clippy --all-targets -- -D warnings
cd web && npm test
bash scripts/tests/build-images.test.sh
bash scripts/tests/dockerfiles.test.sh
bash scripts/tests/publish-images.test.sh
```

## Produktion (Traefik)

Externes Docker-Netz `proxy` (anpassen über `TRAEFIK_NETWORK`).

Images sind `linux/amd64` und liegen in `ghcr.io/logicexception`. Ein Git-Tag-Push startet den Publish-Workflow.

Auf dem Linux-Host (ohne `--build`, sonst wird lokal neu gebaut):

```bash
cp .env.example .env
# TIMEY_HOST, TIMEY_TAG und Passwort setzen
docker compose pull
docker compose up -d
```

Lokaler Dev-Build bleibt `docker compose up -d --build` (Host-Architektur, nicht für den Linux-amd64-Server).

SQLite liegt im Volume `timey-data` unter `/data/timey.db`.

Keycloak/OIDC ist vorbereitet (`auth_provider`, `GET /api/auth/config`), in v1 nicht aktiv.
