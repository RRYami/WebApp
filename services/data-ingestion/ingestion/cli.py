"""CLI entry point for running ingestion pipelines on-demand.

Usage:
    uv run pipeline cpi
    uv run pipeline yield-curve
    uv run pipeline cpi --series-id CPIAUCSL
    uv run pipeline yield-curve --mode parquet
    uv run pipeline yield-curve --mode parquet --parquet-dir ./output
"""

import argparse
import sys
from pathlib import Path
from typing import Callable

import structlog

from ingestion.config import settings
from ingestion.pipelines.cpi import run_cpi_pipeline
from ingestion.pipelines.yield_curve import run_yield_curve_pipeline

logger = structlog.get_logger()

PIPELINES: dict[str, Callable] = {
    "cpi": run_cpi_pipeline,
    "yield-curve": run_yield_curve_pipeline,
}


def main() -> int:
    parser = argparse.ArgumentParser(description="Run a data ingestion pipeline on-demand.")
    parser.add_argument(
        "pipeline",
        choices=list(PIPELINES.keys()),
        help="Pipeline to execute",
    )
    parser.add_argument(
        "--series-id",
        type=str,
        default=None,
        help="Optional FRED series ID override (cpi only)",
    )
    parser.add_argument(
        "--mode",
        choices=["db", "parquet", "both"],
        default="db",
        help="Load mode: db (upsert only), parquet (file only), or both (default: db)",
    )
    parser.add_argument(
        "--parquet-dir",
        type=Path,
        default=Path("data"),
        help="Directory for parquet output files (default: data/)",
    )

    args = parser.parse_args()

    # Only require FRED_API_KEY — active pipelines use FRED exclusively.
    if not settings.fred_api_key:
        logger.error("FRED_API_KEY is required but not configured")
        return 1

    fn = PIPELINES[args.pipeline]
    logger.info("Starting on-demand pipeline", pipeline=args.pipeline, mode=args.mode)

    try:
        kwargs: dict = {
            "mode": args.mode,
            "parquet_dir": args.parquet_dir,
        }
        if args.pipeline == "cpi" and args.series_id:
            kwargs["series_id"] = args.series_id

        loaded = fn(**kwargs)
    except Exception as exc:
        logger.error("Pipeline failed", pipeline=args.pipeline, error=str(exc))
        return 1

    logger.info("Pipeline complete", pipeline=args.pipeline, mode=args.mode, records=loaded)
    return 0


if __name__ == "__main__":
    sys.exit(main())
