# Pricing Platform

A full-stack financial pricing and risk platform for derivatives and fixed income instruments, built with Rust, Python, and React.

## Overview

This project provides a complete quantitative finance stack:

- **pricing-core** — High-performance Rust library for pricing options (Black-Scholes, Barone-Adesi-Whaley, Binomial, Monte Carlo), bonds, and computing Greeks.
- **pricing-api** — Axum-based HTTP API exposing pricing endpoints.
- **data-ingestion** — Python service that ingests market data from FRED and Databento into TimescaleDB.
- **web** — React + Vite + TypeScript frontend.

## Architecture

```
pricing_platform/
├── libs/
│   └── pricing-core/          # Rust quantitative finance library
├── services/
│   ├── pricing-api/           # Rust HTTP API (Axum)
│   └── data-ingestion/        # Python ingestion service
├── web/                       # React frontend
├── infra/
│   ├── docker/                # Docker Compose manifests
│   └── db/migrations/         # TimescaleDB SQL migrations
```

### Tech Stack

| Component | Technology |
|-----------|------------|
| Pricing Engine | Rust, `rust_decimal`, `rayon`, `rand` |
| API | Rust, Axum, Tokio |
| Data Ingestion | Python 3.12, SQLAlchemy, APScheduler, `httpx`, `structlog` |
| Frontend | React 18, TypeScript, Vite |
| Database | TimescaleDB (PostgreSQL 16) |
| Deployment | Docker, Docker Compose, Nginx |

## Getting Started

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- (Optional) [Rust](https://rustup.rs/) 1.78+ for local library development
- (Optional) [uv](https://docs.astral.sh/uv/) for local Python development
- (Optional) [Node.js](https://nodejs.org/) 22+ for local frontend development

### Environment Setup

Copy the example environment file and configure your API keys:

```bash
cp infra/docker/.env.example infra/docker/.env
# Edit infra/docker/.env with your FRED and Databento API keys
```

### Run with Docker Compose

```bash
cd infra/docker
docker compose up --build
```

This starts:

- **TimescaleDB** on port `5432`
- **Pricing API** on port `3000`
- **Data Ingestion** service (scheduler)
- **Web** frontend on port `80`

For development (if available):

```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build
```

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| POST | `/price/european-option` | Price a European option |
| POST | `/price/american-option` | Price an American option |
| POST | `/greeks/european-option` | Compute Greeks for a European option |

Example request:

```bash
curl -X POST http://localhost:3000/price/european-option \
  -H "Content-Type: application/json" \
  -d '{
    "strike": "100",
    "spot": "105",
    "risk_free_rate": "0.05",
    "volatility": "0.2",
    "time_to_maturity": 1.0,
    "option_type": "Call"
  }'
```

## Local Development

### pricing-core (Rust)

```bash
cd libs/pricing-core

# Run tests
cargo test

# Run benchmarks
cargo bench

# Run examples
cargo run --example option_pricing
cargo run --example bond_pricing
cargo run --example american_option_pricing
```

### pricing-api (Rust)

```bash
cd services/pricing-api
cargo run
# API will be available at http://localhost:3000
```

### data-ingestion (Python)

```bash
cd services/data-ingestion

# Install dependencies with uv
uv pip install -e ".[dev]"

# Run the scheduler
python -m ingestion.scheduler

# Format / lint
ruff check .
ruff format .

# Run tests
pytest
```

### web (React)

```bash
cd web
npm install
npm run dev
# Frontend will be available at http://localhost:5173
```

## Database Schema

TimescaleDB stores time-series market data across four hypertables:

- `equity_prices` — OHLCV equity data
- `options_data` — Option quotes, implied volatility, and Greeks
- `yield_curve` — Treasury/yield curve rates by tenor
- `cpi_data` — Consumer Price Index releases

Migrations are applied automatically on container startup via `infra/db/migrations/`.

## Project Structure

```
pricing_platform/
├── libs/pricing-core/
│   ├── src/
│   │   ├── core/          # Money, Currency, InterestRate, DayCount, Errors, Traits
│   │   ├── instruments/   # Bond, Option types
│   │   ├── pricing/       # Black-Scholes, Binomial, Monte Carlo, Barone-Adesi-Whaley
│   │   ├── risk/          # Greeks calculation
│   │   └── utils/
│   ├── examples/          # Runnable examples
│   ├── tests/             # Integration tests
│   ├── benches/           # Criterion benchmarks
│   └── docs/              # Detailed learning documentation
├── services/pricing-api/
│   └── src/
│       ├── main.rs
│       ├── routes.rs
│       ├── handlers.rs
│       └── models.rs
├── services/data-ingestion/
│   └── ingestion/
│       ├── config.py
│       ├── scheduler.py
│       ├── db/
│       ├── pipelines/
│       └── sources/
├── web/
│   └── src/
│       ├── App.tsx
│       ├── main.tsx
│       └── index.css
└── infra/
    ├── docker/
    │   ├── docker-compose.yml
    │   ├── docker-compose.dev.yml
    │   └── .env.example
    └── db/migrations/
```

## Documentation

The `libs/pricing-core/docs/` directory contains detailed learning materials:

1. [Introduction](libs/pricing-core/docs/00-introduction.md)
2. [Overview](libs/pricing-core/docs/01-overview.md)
3. [Core Concepts](libs/pricing-core/docs/02-core-concepts.md)
4. [Traits](libs/pricing-core/docs/03-traits.md)
5. [Error Handling](libs/pricing-core/docs/04-error-handling.md)
6. [Option Pricing](libs/pricing-core/docs/05-option-pricing.md)
7. [Bond Pricing](libs/pricing-core/docs/06-bond-pricing.md)
8. [Testing](libs/pricing-core/docs/07-testing.md)
9. [Rust Patterns](libs/pricing-core/docs/08-rust-patterns.md)
10. [Exercises](libs/pricing-core/docs/09-exercises.md)

## License

MIT OR Apache-2.0
