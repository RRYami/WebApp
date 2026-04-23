"""Yield curve ingestion pipeline via FRED.

Fetches daily Treasury Constant Maturity rates and upserts them
into the yield_curve hypertable.

Example usage:
    from ingestion.pipelines.yield_curve import run_yield_curve_pipeline
    run_yield_curve_pipeline()

Scheduler registration:
    scheduler.add_job(run_yield_curve_pipeline, "cron", hour=16, minute=30)
"""

from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation

import structlog
from sqlalchemy.dialects.postgresql import insert

from ingestion.db.models import YieldCurvePoint
from ingestion.db.session import SessionLocal
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


def _to_decimal(raw: str) -> Decimal | None:
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
) -> int:
    """Fetch Treasury yield curve data from FRED and upsert into TimescaleDB.

    Args:
        tenor_map: Mapping of FRED series IDs to tenor labels.
            Defaults to the full Treasury Constant Maturity curve.

    Returns:
        Total number of records loaded across all tenors.
    """
    tenors = tenor_map or DEFAULT_TENORS
    source = FredSource()
    db = SessionLocal()
    total_loaded = 0

    try:
        for series_id, tenor_label in tenors.items():
            logger.info("Fetching yield curve point", series=series_id, tenor=tenor_label)

            # ---- Fetch & Transform ----
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

            # ---- Normalize to model types ----
            valid_rows: list[dict] = []
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

                valid_rows.append({
                    "curve_date": curve_date,
                    "tenor": tenor_label,
                    "rate": rate,
                    "source": SOURCE_NAME,
                })

            if not valid_rows:
                logger.info(
                    "No valid rows for tenor",
                    series=series_id,
                    tenor=tenor_label,
                )
                continue

            # ---- Bulk Load in chunks ----
            loaded = 0
            chunk_size = 1000
            for i in range(0, len(valid_rows), chunk_size):
                chunk = valid_rows[i : i + chunk_size]
                stmt = (
                    insert(YieldCurvePoint)
                    .values(chunk)
                    .on_conflict_do_nothing(
                        index_elements=["curve_date", "tenor"]
                    )
                )
                result = db.execute(stmt)
                loaded += result.rowcount
                db.commit()

            total_loaded += loaded
            logger.info(
                "Loaded yield curve tenor",
                series=series_id,
                tenor=tenor_label,
                attempted=len(valid_rows),
                loaded=loaded,
                skipped=len(valid_rows) - loaded,
            )

        logger.info(
            "Yield curve pipeline complete",
            total_loaded=total_loaded,
            tenors_processed=len(tenors),
        )
    except Exception as exc:
        db.rollback()
        logger.error("Database load failed", error=str(exc))
        raise
    finally:
        db.close()

    return total_loaded
