from abc import ABC, abstractmethod
from typing import Any


class DataSource(ABC):
    """Abstract base class for all market data sources."""

    name: str = ""

    @abstractmethod
    def fetch(self, **params: Any) -> Any:
        """Fetch raw data from the external source."""
        ...

    @abstractmethod
    def transform(self, raw: Any) -> list[dict]:
        """Transform raw data into standardized records."""
        ...

    @abstractmethod
    def load(self, records: list[dict]) -> None:
        """Load standardized records into the database."""
        ...

    def run(self, **params: Any) -> None:
        """Execute fetch -> transform -> load pipeline."""
        raw = self.fetch(**params)
        records = self.transform(raw)
        self.load(records)
