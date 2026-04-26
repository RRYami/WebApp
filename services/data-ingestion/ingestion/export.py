"""Parquet export utilities for ingestion pipelines.

Writes normalized records to local parquet files with
decimal128 precision preserved and snappy compression.
"""

from datetime import datetime
from decimal import Decimal
from pathlib import Path

import pyarrow as pa
import pyarrow.parquet as pq
import structlog

logger = structlog.get_logger()

# Default decimal precision matching DB schema: Numeric(19, 6)
DEFAULT_DECIMAL_PRECISION = 19
DEFAULT_DECIMAL_SCALE = 6


def _build_schema(records: list[dict]) -> pa.Schema:
    """Infer a PyArrow schema from the first record, preserving decimal types."""
    if not records:
        raise ValueError("Cannot infer schema from empty records")

    sample = records[0]
    fields = []
    for name, value in sample.items():
        if isinstance(value, Decimal):
            fields.append(
                pa.field(name, pa.decimal128(DEFAULT_DECIMAL_PRECISION, DEFAULT_DECIMAL_SCALE))
            )
        elif isinstance(value, datetime):
            fields.append(pa.field(name, pa.timestamp("ns", tz="UTC")))
        elif isinstance(value, bool):
            fields.append(pa.field(name, pa.bool_()))
        elif isinstance(value, int):
            fields.append(pa.field(name, pa.int64()))
        elif isinstance(value, float):
            fields.append(pa.field(name, pa.float64()))
        else:
            fields.append(pa.field(name, pa.string()))

    return pa.schema(fields)


def _convert_records(records: list[dict], schema: pa.Schema) -> dict[str, list]:
    """Convert a list of dicts into columnar arrays matching the schema."""
    columns: dict[str, list] = {field.name: [] for field in schema}
    for rec in records:
        for field in schema:
            val = rec.get(field.name)
            columns[field.name].append(val)
    return columns


def to_parquet(records: list[dict], path: Path) -> Path:
    """Write records to a parquet file with decimal precision preserved.

    Args:
        records: List of normalized dicts from a pipeline.
        path: Output .parquet file path. Parent dirs are created if needed.

    Returns:
        The resolved path of the written file.
    """
    if not records:
        logger.warning("No records to write", path=str(path))
        return path

    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)

    schema = _build_schema(records)
    columns = _convert_records(records, schema)
    table = pa.table(columns, schema=schema)

    pq.write_table(table, path, compression="snappy")

    logger.info(
        "Wrote parquet file",
        path=str(path),
        rows=len(records),
        columns=len(schema),
        size_bytes=path.stat().st_size,
    )
    return path
