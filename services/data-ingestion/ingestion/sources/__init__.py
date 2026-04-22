from ingestion.sources.base import DataSource
from ingestion.sources.databento import DatabentoSource
from ingestion.sources.fred import FredSource

SOURCES: dict[str, type[DataSource]] = {
    "databento": DatabentoSource,
    "fred": FredSource,
}


def get_source(name: str) -> DataSource:
    if name not in SOURCES:
        raise ValueError(f"Unknown source: {name}. Available: {list(SOURCES.keys())}")
    return SOURCES[name]()
