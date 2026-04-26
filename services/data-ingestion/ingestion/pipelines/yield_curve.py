"""Yield curve ingestion pipeline via FRED.

Fetches daily Treasury Constant Maturity rates and upserts them
into the yield_curve hypertable and/or writes to parquet.

Example usage:
    from ingestion.pipelines.yield_curve import run_yield_curve_pipeline
    run_yield_curve_pipeline()

    # Parquet-only (no DB writes):
    run_yield_curve_pipeline(mode="parquet")

Scheduler registration:
    scheduler.add_job(run_yield_curve_pipeline, "cron", hour=16, minute=30)
"""

from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation
from pathlib import Path

import structlog
from sqlalchemy.dialects.postgresql import insert

from ingestion.db.models import YieldCurvePoint
from ingestion.db.session import SessionLocal
from ingestion.export import to_parquet
from ingestion.sources.fred import FredSource

logger = structlog.get_logger()

# FRED series ID -> tenor label mapping for Treasury Constant Maturity rates
DEFAULT_TENORS: dict[str, str] = {
    "DGS1MO": "1M",
    "DGS3MO": "3M",
    "DGS6MO": "6M",
    "DGS1": "1Y",
    "DGS2": "2Y",
    "DGS3": "3Y",
    "DGS5": "5Y",
    "DGS7": "7Y",
    "DGS10": "10Y",
    "DGS20": "20Y",
    "DGS30": "30Y",
}

SOURCE_NAME = "FRED"


def _to_decimal(raw) -> Decimal | None:
    """Convert a FRED value string to Decimal, handling missing values."""
    if raw in (".", "", None):
        return None
    try:
        return Decimal(raw)
    except InvalidOperation:
        logger.warning("Skipping unparseable rate", raw_value=raw)
        return None


def _to_datetime(date_str: str) -> datetime:
    """Parse a YYYY-MM-DD string into a timezone-aware datetime."""
    dt = datetime.strptime(date_str, "%Y-%m-%d")
    return dt.replace(tzinfo=timezone.utc)


def run_yield_curve_pipeline(
    tenor_map: dict[str, str] | None = None,
    mode: str = "db",
    parquet_dir: Path | str = "data",
) -> int:
    """Fetch Treasury yield curve data from FRED and load to DB and/or parquet.

    Args:
        tenor_map: Mapping of FRED series IDs to tenor labels.
            Defaults to the full Treasury Constant Maturity curve.
        mode: One of "db", "parquet", or "both".
        parquet_dir: Base directory for parquet output files.

    Returns:
        Total number of records processed.
    """
    if mode not in ("db", "parquet", "both"):
        raise ValueError(f"Invalid mode: {mode!r}. Must be 'db', 'parquet', or 'both'.")

    tenors = tenor_map or DEFAULT_TENORS
    source = FredSource()

    # ---- Fetch & normalize all tenors into one list ----
    all_rows: list[dict] = []
    for series_id, tenor_label in tenors.items():
        logger.info("Fetching yield curve point", series=series_id, tenor=tenor_label)

        try:
            raw = source.fetch(series_id=series_id)
            records = source.transform(raw)
        except Exception as exc:
            logger.error(
                "Fetch/transform failed",
                series=series_id,
                tenor=tenor_label,
                error=str(exc),
            )
            continue

        for rec in records:
            rate = _to_decimal(rec.get("value"))
            if rate is None:
                continue

            try:
                curve_date = _to_datetime(rec["release_date"])
            except (KeyError, ValueError) as exc:
                logger.warning(
                    "Skipping record with bad date",
                    series=series_id,
                    record=rec,
                    error=str(exc),
                )
                continue

            all_rows.append(
                {
                    "curve_date": curve_date,
                    "tenor": tenor_label,
                    "rate": rate,
                    "source": SOURCE_NAME,
                }
            )

    if not all_rows:
        logger.info("No valid rows across all tenors")
        return 0

    logger.info("Normalized all tenors", total_rows=len(all_rows), tenors_processed=len(tenors))

    # ---- Write parquet ----
    if mode in ("parquet", "both"):
        parquet_path = Path(parquet_dir) / "yield_curve.parquet"
        to_parquet(all_rows, parquet_path)

    # ---- DB upsert ----
    if mode in ("db", "both"):
        db = SessionLocal()
        loaded = 0
        chunk_size = 1000
        try:
            for i in range(0, len(all_rows), chunk_size):
                chunk = all_rows[i : i + chunk_size]
                stmt = (
                    insert(YieldCurvePoint)
                    .values(chunk)
                    .on_conflict_do_nothing(index_elements=["curve_date", "tenor"])
                )
                result = db.execute(stmt)
                loaded += result.rowcount  # type: ignore
                db.commit()

            logger.info(
                "Yield curve DB upsert complete",
                attempted=len(all_rows),
                loaded=loaded,
                skipped=len(all_rows) - loaded,
            )
        except Exception as exc:
            db.rollback()
            logger.error("Database load failed", error=str(exc))
            raise
        finally:
            db.close()

    return len(all_rows)
