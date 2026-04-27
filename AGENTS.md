# Agent Guide: Pricing Platform

This file contains project-specific instructions for AI agents working on this codebase.

## Project Summary

A full-stack quantitative finance platform consisting of:
- `libs/pricing-core`: Rust library for option/bond pricing and risk (Greeks).
- `services/pricing-api`: Axum HTTP API wrapping `pricing-core`.
- `services/data-ingestion`: Python service ingesting market data into TimescaleDB.
- `web`: React + Vite + TypeScript frontend.
- `infra/docker`: Docker Compose orchestration.
- `infra/db/migrations`: TimescaleDB SQL migrations.

## Build & Test Commands

### Rust (pricing-core, pricing-api)
- **Build**: `cargo build`
- **Build release**: `cargo build --release`
- **Test**: `cargo test`
- **Test all workspaces**: `cargo test --workspace`
- **Benchmarks**: `cargo bench`
- **Run examples**:
  - `cargo run --example option_pricing`
  - `cargo run --example bond_pricing`
  - `cargo run --example american_option_pricing`
- **Clippy**: `cargo clippy --all-targets --all-features`
- **Fmt**: `cargo fmt`
- **MSRV**: Rust 1.78

### Python (data-ingestion)
- **Install**: `uv pip install -e ".[dev]"`
- **Run scheduler**: `python -m ingestion.scheduler`
- **Run pipeline on-demand**: `uv run pipeline cpi` or `uv run pipeline yield-curve`
- **Lint**: `ruff check .`
- **Format**: `ruff format .`
- **Test**: `pytest` (skips integration tests by default)
- **Integration tests**: `pytest -m integration` (requires live FRED_API_KEY)
- **Line length**: 100

### Web (React/TypeScript)
- **Install**: `npm install`
- **Dev**: `npm run dev`
- **Build**: `npm run build`
- **Preview**: `npm run preview`

### Docker
- **Start all**: `cd infra/docker && docker compose --env-file ../../.env up --build`
- **Dev override**: `cd infra/docker && docker compose -f docker-compose.yml -f docker-compose.dev.yml --env-file ../../.env up --build`

## Architecture & Conventions

### Rust
- Use `rust_decimal` and `rust_decimal_macros::dec!` for all monetary/financial arithmetic. **Never use `f64` for money.**
- The core error type is `pricing_core::Error` (defined in `core/error.rs`). Use `Result<T>` alias.
- Key traits: `Pricable` (`.price()`), `HasGreeks` (`.greeks()`), `Instrument`.
- `pricing-core` is a workspace library crate. `pricing-api` depends on it with the `serde` feature enabled.
- Prefer `thiserror` for error variants.
- Use `rayon` for parallel Monte Carlo simulations.
- Module layout:
  - `core/` — foundational types (Money, Currency, InterestRate, DayCountConvention, errors, traits)
  - `instruments/` — Bond, EuropeanOption, AmericanOption
  - `pricing/` — BlackScholes, BinomialModel, BaroneAdesiWhaley, MonteCarlo, EngineRegistry
  - `risk/` — Greeks

### Python
- Uses Pydantic Settings (`ingestion/config.py`) for env-based config.
- SQLAlchemy 2.0 models in `ingestion/db/models.py`.
- Scheduler entry point: `ingestion/scheduler.py` (APScheduler).
- Data sources: `ingestion/sources/fred.py`, `ingestion/sources/databento.py`.
- Pipelines: `ingestion/pipelines/equities.py`, `ingestion/pipelines/options.py`, `ingestion/pipelines/yield_curve.py`, `ingestion/pipelines/cpi.py`.
- Logging: `ingestion/logging_config.py` sets up `structlog` bridged to stdlib `logging`. Console output is plain text (INFO+); file output is JSON Lines via `RotatingFileHandler` (DEBUG+, 5 MB rotation, 3 backups). Entry points must call `setup_logging(settings.log_level, settings.log_file_path)` before any log emissions.
- Keep line length ≤ 100.

### Database
- TimescaleDB (PostgreSQL 16) with automatic hypertable creation via migrations.
- Tables: `equity_prices`, `options_data`, `yield_curve`, `cpi_data`.
- Connection string is passed via `DATABASE_URL` env var.

### Frontend
- React 18 with TypeScript, built with Vite.
- Currently a minimal scaffold; API calls should target the `pricing-api` service.
- Nginx serves the built static files in production.

## API Contract

The pricing-api exposes these JSON endpoints:

- `POST /price/european-option`
  - Body: `{ strike, spot, risk_free_rate, volatility, time_to_maturity, option_type }`
  - Response: `{ price, currency }`

- `POST /price/american-option`
  - Body: `{ strike, spot, risk_free_rate, volatility, time_to_maturity, option_type, dividend_yield? }`
  - Response: `{ price, currency }`

- `POST /greeks/european-option`
  - Body: same as european-option pricing
  - Response: `{ delta, gamma, theta, vega, rho, phi }`

## Environment Variables

Copy `.env.example` to `.env` at the project root and set:

- `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB`
- `DATABASE_URL`
- `FRED_API_KEY`
- `DATABENTO_API_KEY`
- `LOG_LEVEL`
- `LOG_FILE_PATH` (default: `services/data-ingestion/logs/app.log.jsonl`)

> `.env` is gitignored. Never commit real secrets.

## Important Notes for Agents

1. **Decimal precision**: All financial calculations in Rust must use `Decimal`. Floating-point is unacceptable for money.
2. **Feature flags**: `pricing-core` has an optional `serde` feature. Enable it when the consumer needs serialization (e.g., `pricing-api`).
3. **Migrations**: SQL files in `infra/db/migrations/` run in lexicographic order on container startup. Name new migrations with zero-padded prefixes (e.g., `006_...sql`).
4. **Adding a new instrument**: Implement `Instrument` + `Pricable` (+ `HasGreeks` if applicable). Add to `prelude` and re-export in `lib.rs`.
5. **Adding a new pipeline**: Create a pipeline in `ingestion/pipelines/`, register it in `ingestion/scheduler.py`.
6. **API handlers**: Keep HTTP logic in `handlers.rs`, routing in `routes.rs`, and request/response DTOs in `models.rs`.
7. **Tests**: Add unit tests next to the code, integration tests in `tests/`, and benchmarks in `benches/`.
8. **Documentation**: The `libs/pricing-core/docs/` directory is for learning materials. Do not delete or move these files unless explicitly asked.
9. **Docker builds**: The Rust API Dockerfile copies from the repo root context so it can access `libs/pricing-core`. The Python and web Dockerfiles use their own directories as context.
10. **Docker env file**: Always run compose with `--env-file ../../.env` from `infra/docker/`. The `.env` must live at the repo root.
11. **Pipeline bulk inserts**: Use chunked bulk inserts (e.g., 1,000 rows per commit) for TimescaleDB hypertables to avoid `out of shared memory` errors.
12. **Logging initialization**: `setup_logging()` must be called once at every Python entry point (`cli.py`, `scheduler.py`, tests). Never configure logging at import time.
13. **Git safety**: Do not run `git commit`, `git push`, or destructive git commands unless explicitly requested.
