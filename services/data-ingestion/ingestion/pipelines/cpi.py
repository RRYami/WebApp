"""CPI data ingestion pipeline via FRED.

Example usage:
    from ingestion.pipelines.cpi import run_cpi_pipeline
    run_cpi_pipeline()

Scheduler registration:
    scheduler.add_job(run_cpi_pipeline, "cron", day="15", hour=9, minute=0)
"""

from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation

import structlog
from sqlalchemy.dialects.postgresql import insert

from ingestion.db.models import CpiData
from ingestion.db.session import SessionLocal
from ingestion.sources.fred import FredSource

logger = structlog.get_logger()

# FRED series ID for US CPI All Urban Consumers, Seasonally Adjusted
FRED_CPI_SERIES = "CPIAUCSL"


def _to_decimal(raw: str) -> Decimal | None:
    """Convert a FRED value string to Decimal, handling missing values."""
    if raw in (".", "", None):
        return None
    try:
        return Decimal(raw)
    except InvalidOperation:
        logger.warning("Skipping unparseable value", raw_value=raw)
        return None


def _to_datetime(date_str: str) -> datetime:
    """Parse a YYYY-MM-DD string into a timezone-aware datetime."""
    dt = datetime.strptime(date_str, "%Y-%m-%d")
    return dt.replace(tzinfo=timezone.utc)


def run_cpi_pipeline(series_id: str = FRED_CPI_SERIES) -> int:
    """Fetch CPI observations from FRED and upsert into TimescaleDB.

    Args:
        series_id: FRED series ID to ingest (default: CPIAUCSL).

    Returns:
        Number of records loaded.
    """
    source = FredSource()
    logger.info("Starting CPI pipeline", series=series_id)

    # ---- Fetch & Transform ----
    try:
        raw = source.fetch(series_id=series_id)
        records = source.transform(raw)
    except Exception as exc:
        logger.error("Fetch/transform failed", series=series_id, error=str(exc))
        raise

    if not records:
        logger.info("No records returned from source", series=series_id)
        return 0

    # ---- Normalize to model types ----
    valid_rows: list[dict] = []
    for rec in records:
        value = _to_decimal(rec.get("value"))
        if value is None:
            continue

        try:
            release_date = _to_datetime(rec["release_date"])
        except (KeyError, ValueError) as exc:
            logger.warning("Skipping record with bad date", record=rec, error=str(exc))
            continue

        valid_rows.append({
            "release_date": release_date,
            "series_id": rec.get("series_id", series_id),
            "value": value,
            "period": rec.get("period", ""),
        })

    if not valid_rows:
        logger.info("No valid rows after filtering", series=series_id)
        return 0

    # ---- Load ----
    db = SessionLocal()
    loaded = 0
    try:
        for row in valid_rows:
            stmt = (
                insert(CpiData)
                .values(**row)
                .on_conflict_do_nothing(
                    index_elements=["release_date", "series_id"]
                )
            )
            result = db.execute(stmt)
            loaded += result.rowcount  # 1 if inserted, 0 if conflict

        db.commit()
        logger.info(
            "CPI pipeline complete",
            series=series_id,
            attempted=len(valid_rows),
            loaded=loaded,
            skipped=len(valid_rows) - loaded,
        )
    except Exception as exc:
        db.rollback()
        logger.error("Database load failed", series=series_id, error=str(exc))
        raise
    finally:
        db.close()

    return loaded
