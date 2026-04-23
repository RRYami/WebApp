"""CLI entry point for running ingestion pipelines on-demand.

Usage:
    uv run pipeline cpi
    uv run pipeline yield-curve
    uv run pipeline cpi --series-id CPIAUCSL
"""

import argparse
import sys

import structlog

from ingestion.config import settings
from ingestion.pipelines.cpi import run_cpi_pipeline
from ingestion.pipelines.yield_curve import run_yield_curve_pipeline

logger = structlog.get_logger()

PIPELINES: dict[str, callable] = {
    "cpi": run_cpi_pipeline,
    "yield-curve": run_yield_curve_pipeline,
}


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run a data ingestion pipeline on-demand."
    )
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

    args = parser.parse_args()

    settings.validate_secrets()

    fn = PIPELINES[args.pipeline]
    logger.info("Starting on-demand pipeline", pipeline=args.pipeline)

    try:
        if args.pipeline == "cpi" and args.series_id:
            loaded = fn(series_id=args.series_id)
        else:
            loaded = fn()
    except Exception as exc:
        logger.error("Pipeline failed", pipeline=args.pipeline, error=str(exc))
        return 1

    logger.info("Pipeline complete", pipeline=args.pipeline, loaded=loaded)
    return 0


if __name__ == "__main__":
    sys.exit(main())
