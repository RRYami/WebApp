"""Scheduler entry point for data ingestion."""

import structlog
from apscheduler.schedulers.blocking import BlockingScheduler

from ingestion.config import settings
from ingestion.pipelines.cpi import run_cpi_pipeline
from ingestion.pipelines.yield_curve import run_yield_curve_pipeline

logger = structlog.get_logger()


def main():
    scheduler = BlockingScheduler()

    # CPI is typically released monthly by the BLS around the 10th–15th.
    scheduler.add_job(
        run_cpi_pipeline,
        "cron",
        day="10-15",
        hour=9,
        minute=0,
        id="cpi_pipeline",
        replace_existing=True,
    )

    # Treasury yields are updated daily on FRED after US market close.
    scheduler.add_job(
        run_yield_curve_pipeline,
        "cron",
        hour=16,
        minute=30,
        id="yield_curve_pipeline",
        replace_existing=True,
    )

    logger.info("Starting data ingestion scheduler", log_level=settings.log_level)
    scheduler.start()


if __name__ == "__main__":
    main()
