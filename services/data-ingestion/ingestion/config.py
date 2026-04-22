from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict

# Resolve the project root relative to this file:
# ingestion/config.py -> services/data-ingestion/ingestion -> services/data-ingestion -> services -> root
_PROJECT_ROOT = Path(__file__).resolve().parents[3]


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=str(_PROJECT_ROOT / ".env"),
        env_file_encoding="utf-8",
        extra="ignore",
    )

    database_url: str = "postgresql://postgres:postgres@localhost:5432/pricing"
    databento_api_key: str | None = None
    fred_api_key: str | None = None
    log_level: str = "INFO"

    def validate_secrets(self) -> None:
        """Fail fast if required API keys are missing."""
        missing = []
        if not self.fred_api_key:
            missing.append("FRED_API_KEY")
        if not self.databento_api_key:
            missing.append("DATABENTO_API_KEY")
        if missing:
            raise RuntimeError(
                f"Missing required API keys: {', '.join(missing)}. "
                f"Ensure they are set in {_PROJECT_ROOT / '.env'} or in your environment."
            )


settings = Settings()
