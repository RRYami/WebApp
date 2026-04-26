"""CPI data ingestion pipeline via FRED.

Example usage:
    from ingestion.pipelines.cpi import run_cpi_pipeline
    run_cpi_pipeline()

    # Parquet-only (no DB writes):
    run_cpi_pipeline(mode="parquet")

Scheduler registration:
    scheduler.add_job(run_cpi_pipeline, "cron", day="15", hour=9, minute=0)
"""

from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation
from pathlib import Path

import structlog
from sqlalchemy.dialects.postgresql import insert

from ingestion.db.models import CpiData
from ingestion.db.session import SessionLocal
from ingestion.export import to_parquet
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


def run_cpi_pipeline(
    series_id: str = FRED_CPI_SERIES,
    mode: str = "db",
    parquet_dir: Path | str = "data",
) -> int:
    """Fetch CPI observations from FRED and load to DB and/or parquet.

    Args:
        series_id: FRED series ID to ingest (default: CPIAUCSL).
        mode: One of "db", "parquet", or "both".
        parquet_dir: Base directory for parquet output files.

    Returns:
        Number of records processed.
    """
    if mode not in ("db", "parquet", "both"):
        raise ValueError(f"Invalid mode: {mode!r}. Must be 'db', 'parquet', or 'both'.")

    source = FredSource()
    logger.info("Starting CPI pipeline", series=series_id, mode=mode)

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

        valid_rows.append(
            {
                "release_date": release_date,
                "series_id": rec.get("series_id", series_id),
                "value": value,
                "period": rec.get("period", ""),
            }
        )

    if not valid_rows:
        logger.info("No valid rows after filtering", series=series_id)
        return 0

    logger.info("Normalized CPI records", rows=len(valid_rows), series=series_id)

    # ---- Write parquet ----
    if mode in ("parquet", "both"):
        parquet_path = Path(parquet_dir) / "cpi.parquet"
        to_parquet(valid_rows, parquet_path)

    # ---- DB upsert ----
    if mode in ("db", "both"):
        db = SessionLocal()
        loaded = 0
        chunk_size = 1000
        try:
            for i in range(0, len(valid_rows), chunk_size):
                chunk = valid_rows[i : i + chunk_size]
                stmt = (
                    insert(CpiData)
                    .values(chunk)
                    .on_conflict_do_nothing(index_elements=["release_date", "series_id"])
                )
                result = db.execute(stmt)
                loaded += result.rowcount
                db.commit()

            logger.info(
                "CPI DB upsert complete",
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

    return len(valid_rows)
