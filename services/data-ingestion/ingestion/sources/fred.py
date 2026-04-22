import httpx

from ingestion.config import settings
from ingestion.sources.base import DataSource


class FredSource(DataSource):
    """FRED (Federal Reserve Economic Data) API source."""

    name = "fred"
    BASE_URL = "https://api.stlouisfed.org/fred"

    def __init__(self) -> None:
        self.api_key = settings.fred_api_key

    def fetch(self, **params) -> dict:
        if not self.api_key:
            raise RuntimeError("FRED_API_KEY is not configured")
        series_id = params.get("series_id")
        url = f"{self.BASE_URL}/series/observations"
        query = {
            "series_id": series_id,
            "api_key": self.api_key,
            "file_type": "json",
        }
        response = httpx.get(url, params=query, timeout=30.0)
        response.raise_for_status()
        return response.json()

    def transform(self, raw: dict) -> list[dict]:
        series_id = raw.get("series_id", "UNKNOWN")
        observations = raw.get("observations", [])
        records = []
        for obs in observations:
            records.append({
                "series_id": series_id,
                "release_date": obs["date"],
                "value": obs["value"],
                "period": obs.get("period", ""),
            })
        return records

    def load(self, records: list[dict]) -> None:
        # TODO: implement batch insert via SQLAlchemy
        pass
