# career

A basic Rust web API.

## Stack

- Rust (edition 2024), tokio, axum
- Postgres 18 with SeaORM
- SeaORM migrations in the `migration` crate
- `tower-http` request tracing
- Docker Compose to run the full stack

## Start the stack

```bash
docker compose up --build
```

The API listens on port 3000. To use a different host port, set `API_PORT`:

```bash
API_PORT=3001 docker compose up --build
```

Check that the API runs:

```bash
curl -i localhost:3000/health
```

The endpoint returns `200` and `{"status":"ok","db":"up"}` when the database is reachable. It returns `503` when the database is down.

## Environment variables

The API reads these in `src/config.rs`. It also loads a `.env` file if one exists. See `.env.example`.

| Name | Default | Use |
| --- | --- | --- |
| `DATABASE_URL` | none (required) | Postgres connection string |
| `BIND_ADDR` | `0.0.0.0:3000` | Address the API binds to |
| `DB_MAX_CONNECTIONS` | `10` | Maximum size of the database connection pool |
| `RUST_LOG` | `info,tower_http=debug` | Log filter |
| `API_PORT` | `3000` | Host port that Docker Compose publishes |

## Run the API on the host

You need a Rust toolchain and a macOS or Linux C toolchain. Start only the database:

```bash
docker compose up -d db
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/career
cargo run
```

The API applies pending migrations when it starts.

## Migrations

There are no tables yet. The migration list is in `migration/src/lib.rs`.

Install the SeaORM CLI one time:

```bash
cargo install sea-orm-cli
```

Create a migration:

```bash
sea-orm-cli migrate generate create_example_table
```

Add the new migration to `Migrator::migrations()` in `migration/src/lib.rs`. Then run the migrations:

```bash
sea-orm-cli migrate up
```

`sea-orm-cli` reads `DATABASE_URL` from the environment.

## Tests

The tests use a mock database. They do not need Postgres.

```bash
cargo test --workspace
```

## Git hooks

The repository has Git hooks in `.githooks/`. Turn them on one time in each clone:

```bash
git config core.hooksPath .githooks
```

- `pre-commit` runs `cargo fmt --all --check`.
- `pre-push` runs `cargo clippy --workspace --all-targets -- -D warnings`.

These are the same checks that CI runs on a pull request. To skip a hook one time, add `--no-verify` to the Git command.

## Layout

- `src/config.rs`: configuration struct, read from the environment
- `src/lib.rs`: router, handlers, and state
- `src/main.rs`: start-up, database connection, and server
- `migration/`: migration crate and its CLI binary
- `tests/`: integration tests
- `.githooks/`: Git hooks for format and lint checks
- `Dockerfile`: multi-stage build with cargo-chef dependency caching
