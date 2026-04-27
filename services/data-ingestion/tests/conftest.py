import logging

import pytest

from ingestion.logging_config import setup_logging


@pytest.fixture(autouse=True)
def _reset_logging():
    """Reset logging between tests so dictConfig calls don't stack handlers."""
    root = logging.getLogger()
    for handler in root.handlers[:]:
        root.removeHandler(handler)
        handler.close()
    yield
    for handler in root.handlers[:]:
        root.removeHandler(handler)
        handler.close()


@pytest.fixture()
def tmp_log_dir(tmp_path):
    """Return a temporary log file path and ensure cleanup."""
    log_file = tmp_path / "logs" / "test.log.jsonl"
    return str(log_file)


@pytest.fixture()
def configured_logging(tmp_log_dir):
    """Set up logging with a temp file path and return the path."""
    setup_logging("DEBUG", tmp_log_dir)
    return tmp_log_dir
