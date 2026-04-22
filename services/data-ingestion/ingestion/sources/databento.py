from ingestion.sources.base import DataSource


class DatabentoSource(DataSource):
    """Databento API source for options and equities data."""

    name = "databento"
    BASE_URL = "https://hist.databento.com/v0"

    def __init__(self) -> None:
        self.api_key = None  # TODO: configure from settings

    def fetch(self, **params) -> dict:
        # TODO: implement Databento API fetch
        raise NotImplementedError("Databento fetch not yet implemented")

    def transform(self, raw: dict) -> list[dict]:
        # TODO: implement transform
        raise NotImplementedError("Databento transform not yet implemented")

    def load(self, records: list[dict]) -> None:
        # TODO: implement batch insert via SQLAlchemy
        pass
