"""Scheduler entry point for data ingestion."""

from apscheduler.schedulers.blocking import BlockingScheduler

from ingestion.config import settings
import structlog

logger = structlog.get_logger()


def main():
    scheduler = BlockingScheduler()
    # TODO: register pipeline jobs here
    # Example:
    # scheduler.add_job(run_cpi_pipeline, "cron", hour=9, minute=0)
    logger.info("Starting data ingestion scheduler", log_level=settings.log_level)
    scheduler.start()


if __name__ == "__main__":
    main()
