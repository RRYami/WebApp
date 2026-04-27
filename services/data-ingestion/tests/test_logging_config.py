"""Tests for the logging configuration module."""

import json
import logging
import logging.handlers

import pytest
import structlog

from ingestion.logging_config import JsonFormatter, setup_logging


class TestJsonFormatter:
    def test_produces_valid_json(self):
        formatter = JsonFormatter()
        record = logging.LogRecord(
            name="test.logger",
            level=logging.INFO,
            pathname="test.py",
            lineno=42,
            msg="hello world",
            args=(),
            exc_info=None,
        )
        output = formatter.format(record)
        data = json.loads(output)

        assert data["message"] == "hello world"
        assert data["level"] == "INFO"
        assert data["logger"] == "test.logger"
        assert data["line"] == 42
        assert data["module"] == "test"
        assert data["function"] is None
        assert data["thread"] == "MainThread"
        assert data["process"] == "MainProcess"

    def test_includes_timestamp(self):
        formatter = JsonFormatter()
        record = logging.LogRecord(
            name="test",
            level=logging.INFO,
            pathname="test.py",
            lineno=1,
            msg="msg",
            args=(),
            exc_info=None,
        )
        data = json.loads(formatter.format(record))
        assert "timestamp" in data
        assert len(data["timestamp"]) > 0

    def test_includes_extra_context(self):
        formatter = JsonFormatter()
        record = logging.LogRecord(
            name="test",
            level=logging.INFO,
            pathname="test.py",
            lineno=1,
            msg="fetching data",
            args=(),
            exc_info=None,
        )
        record.series = "CPIAUCSL"
        record.records = 150
        data = json.loads(formatter.format(record))

        assert data["series"] == "CPIAUCSL"
        assert data["records"] == 150

    def test_includes_exception_traceback(self):
        formatter = JsonFormatter()
        try:
            raise ValueError("boom")
        except ValueError:
            import sys

            exc_info = sys.exc_info()
        record = logging.LogRecord(
            name="test",
            level=logging.ERROR,
            pathname="test.py",
            lineno=1,
            msg="failed",
            args=(),
            exc_info=exc_info,
        )
        output = formatter.format(record)
        data = json.loads(output)

        assert "exception" in data
        assert "ValueError: boom" in data["exception"]

    def test_all_base_keys_present(self):
        formatter = JsonFormatter()
        record = logging.LogRecord(
            name="ingestion.pipelines.cpi",
            level=logging.WARNING,
            pathname="cpi.py",
            lineno=99,
            msg="skipping value",
            args=(),
            exc_info=None,
        )
        data = json.loads(formatter.format(record))

        expected_keys = {
            "timestamp",
            "level",
            "logger",
            "message",
            "module",
            "function",
            "line",
            "thread",
            "process",
        }
        assert expected_keys.issubset(data.keys())


class TestSetupLogging:
    def test_console_handler_configured(self, configured_logging):
        root = logging.getLogger()
        handler_types = [type(h).__name__ for h in root.handlers]
        assert "StreamHandler" in handler_types

    def test_file_handler_configured(self, configured_logging):
        root = logging.getLogger()
        handler_types = [type(h).__name__ for h in root.handlers]
        assert "RotatingFileHandler" in handler_types

    def test_root_level_set(self, configured_logging):
        root = logging.getLogger()
        assert root.level == logging.DEBUG

    def test_log_file_created(self, configured_logging):
        import os

        assert os.path.exists(configured_logging)

    def test_structlog_configured(self, configured_logging):
        logger = structlog.get_logger("test_structlog")
        assert isinstance(logger._context, dict)

    def test_structlog_context_reaches_file(self, configured_logging):
        logger = structlog.get_logger("test.context")
        logger.info("structured message", series="CPIAUCSL", record_count=42)

        for handler in logging.getLogger().handlers:
            if isinstance(handler, logging.handlers.RotatingFileHandler):
                handler.flush()
                with open(handler.baseFilename) as f:
                    lines = f.readlines()
                break
        else:
            pytest.skip("No file handler found")

        last_line = lines[-1].strip()
        data = json.loads(last_line)

        assert data["logger"] == "test.context"
        assert data["message"] == "structured message"
        assert data["series"] == "CPIAUCSL"
        assert data["record_count"] == 42

    def test_noisy_loggers_suppressed_at_info(self, tmp_log_dir):
        setup_logging("INFO", tmp_log_dir)
        assert logging.getLogger("apscheduler").level == logging.WARNING
        assert logging.getLogger("sqlalchemy.engine").level == logging.WARNING

    def test_noisy_loggers_not_suppressed_at_debug(self, configured_logging):
        assert logging.getLogger("apscheduler").level == logging.WARNING

    def test_console_only_when_no_file_path(self):
        setup_logging("INFO", log_file_path=None)
        root = logging.getLogger()
        handler_types = [type(h).__name__ for h in root.handlers]
        assert "StreamHandler" in handler_types
        assert "RotatingFileHandler" not in handler_types
