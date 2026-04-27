from pathlib import Path

from pydantic_settings import BaseSettings, SettingsConfigDict

# Resolve the project root relative to this file:
# ingestion/config.py -> services/data-ingestion/ingestion
# -> services/data-ingestion -> services -> root
_config_file = Path(__file__).resolve()
try:
    _PROJECT_ROOT = _config_file.parents[3]
except IndexError:
    # Fallback for Docker where the file is at /app/ingestion/config.py
    _PROJECT_ROOT = _config_file.parents[-1]

_ENV_FILE = _PROJECT_ROOT / ".env"


class Settings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=str(_ENV_FILE) if _ENV_FILE.exists() else None,
        env_file_encoding="utf-8",
        extra="ignore",
    )

    database_url: str = "postgresql://postgres:postgres@localhost:5432/pricing"
    databento_api_key: str | None = None
    fred_api_key: str | None = None
    log_level: str = "INFO"
    log_file_path: str = str(_config_file.parent.parent / "logs" / "app.log.jsonl")

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
