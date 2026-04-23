"""Integration tests for the FRED data source.

These tests hit the live FRED API and require a valid FRED_API_KEY.
Run with: uv run pytest -m integration
Skip by default: uv run pytest
"""

import pytest

from ingestion.config import settings
from ingestion.sources.fred import FredSource

pytestmark = pytest.mark.integration


@pytest.fixture(scope="module")
def fred_api_key():
    """Validate FRED_API_KEY is present; skip integration tests otherwise."""
    key = settings.fred_api_key
    if not key:
        pytest.skip("FRED_API_KEY not configured in .env")
    return key


@pytest.fixture
def source() -> FredSource:
    """Return an initialized FredSource."""
    return FredSource()


class TestFredSourceFetch:
    def test_fetch_returns_observations(self, source: FredSource) -> None:
        """FredSource.fetch must return a dict containing observations."""
        raw = source.fetch(series_id="CPIAUCSL")

        assert isinstance(raw, dict)
        assert "observations" in raw
        assert isinstance(raw["observations"], list)
        assert len(raw["observations"]) > 0

    def test_fetch_observation_structure(self, source: FredSource) -> None:
        """Each observation must contain the expected FRED fields."""
        raw = source.fetch(series_id="CPIAUCSL")
        obs = raw["observations"][0]

        assert "date" in obs
        assert "value" in obs
        # FRED dates are YYYY-MM-DD strings
        assert isinstance(obs["date"], str)
        assert len(obs["date"]) == 10

    def test_fetch_response_structure(self, source: FredSource) -> None:
        """The raw payload must contain observations and metadata keys."""
        raw = source.fetch(series_id="CPIAUCSL")

        assert "observations" in raw
        assert "count" in raw
        assert isinstance(raw["count"], int)
        assert raw["count"] > 0


class TestFredSourceTransform:
    def test_transform_returns_list_of_dicts(self, source: FredSource) -> None:
        """Transform must normalize observations into a list of dicts."""
        raw = source.fetch(series_id="CPIAUCSL")
        records = source.transform(raw)

        assert isinstance(records, list)
        assert len(records) > 0
        for rec in records:
            assert isinstance(rec, dict)

    def test_transform_record_schema(self, source: FredSource) -> None:
        """Each transformed record must have the keys pipelines expect."""
        raw = source.fetch(series_id="CPIAUCSL")
        records = source.transform(raw)
        rec = records[0]

        required_keys = {"series_id", "release_date", "value", "period"}
        assert required_keys.issubset(rec.keys())

    def test_transform_series_id_populated(self, source: FredSource) -> None:
        """The series_id field should be set, not UNKNOWN."""
        raw = source.fetch(series_id="CPIAUCSL")
        records = source.transform(raw)

        for rec in records:
            assert rec["series_id"] == "CPIAUCSL"


class TestFredSourceEndToEnd:
    def test_small_series_round_trip(self, source: FredSource) -> None:
        """Use a small FRED series (1-Month Treasury) for a fast E2E test."""
        raw = source.fetch(series_id="DGS1MO")
        records = source.transform(raw)

        assert len(records) > 0
        for rec in records:
            assert rec["series_id"] == "DGS1MO"
            assert rec["release_date"]
            assert rec["value"] is not None or rec["value"] == "."
