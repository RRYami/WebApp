"""Centralised logging configuration for the data-ingestion service.

Initialises structlog + stdlib logging so that:

* Console (stdout) prints human-readable plain text at INFO and above.
* File handler writes JSON Lines to a rotating file at DEBUG and above.
* structlog key-value pairs (e.g. ``logger.info("msg", key="val")``) are
  merged into the JSON output automatically.
* SQLAlchemy and APScheduler noise is suppressed unless DEBUG is requested.
* The ``LOG_LEVEL`` and ``LOG_FILE_PATH`` settings are wired in from
  ``ingestion.config.Settings``.
"""

import json
import logging
import logging.config
import logging.handlers
from pathlib import Path
from typing import override

import structlog

_STD_ATTRS = frozenset(
    {
        "name",
        "msg",
        "args",
        "levelname",
        "levelno",
        "pathname",
        "filename",
        "module",
        "exc_info",
        "exc_text",
        "stack_info",
        "lineno",
        "funcName",
        "created",
        "msecs",
        "relativeCreated",
        "thread",
        "threadName",
        "process",
        "processName",
        "taskName",
        "message",
        "asctime",
    }
)

_BASE_JSON_KEYS = {
    "level",
    "timestamp",
    "logger",
    "message",
    "module",
    "function",
    "line",
    "thread",
    "process",
}


class JsonFormatter(logging.Formatter):
    """Produce DataProject-compatible JSON Lines.

    Standard fields match DataProject's ``fmt_keys`` mapping:

    .. code-block:: json

        {
          "timestamp": "2026-04-27 14:32:01",
          "level": "INFO",
          "logger": "ingestion.pipelines.cpi",
          "message": "Starting CPI pipeline",
          "module": "cpi",
          "function": "run_cpi_pipeline",
          "line": 42,
          "thread": "MainThread",
          "process": "MainProcess",
          "series": "CPIAUCSL"   <-- auto-merged structlog context
        }
    """

    def __init__(self, datefmt: str = "%Y-%m-%d %H:%M:%S", **kwargs):
        super().__init__(datefmt=datefmt, **kwargs)

    @override
    def format(self, record: logging.LogRecord) -> str:
        log_record: dict = {
            "timestamp": self.formatTime(record, self.datefmt),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
            "line": record.lineno,
            "thread": record.threadName,
            "process": record.processName,
        }

        for key, value in record.__dict__.items():
            if key not in _STD_ATTRS and key not in _BASE_JSON_KEYS:
                log_record[key] = value

        if record.exc_info and record.exc_text is None:
            record.exc_text = self.formatException(record.exc_info)
        if record.exc_text:
            log_record["exception"] = record.exc_text

        return json.dumps(log_record, default=str)


def setup_logging(log_level: str = "INFO", log_file_path: str | None = None) -> None:
    """Configure structlog + stdlib logging.  Call once at application start.

    Parameters
    ----------
    log_level:
        Root logger level (e.g. ``"DEBUG"``, ``"INFO"``).  Read from
        ``settings.log_level`` in normal use.
    log_file_path:
        Path to the JSON Lines log file.  When ``None`` (the default) the
        file handler is skipped and only console output is configured.
    """
    log_level = log_level.upper()

    handlers: list[str] = ["console"]
    handler_config: dict = {
        "console": {
            "class": "logging.StreamHandler",
            "level": "INFO",
            "formatter": "plain",
            "stream": "ext://sys.stdout",
        },
    }

    if log_file_path:
        log_path = Path(log_file_path)
        log_path.parent.mkdir(parents=True, exist_ok=True)

        handlers.append("file")
        handler_config["file"] = {
            "class": "logging.handlers.RotatingFileHandler",
            "level": "DEBUG",
            "formatter": "json",
            "filename": str(log_path),
            "maxBytes": 5_000_000,
            "backupCount": 3,
        }

        # Silence RotatingFileHandler's "you must specify a name" log if
        # it fires before dictConfig fully initialises the handler.
        logging.getLogger("logging.handlers").setLevel(logging.WARNING)

    config = {
        "version": 1,
        "disable_existing_loggers": False,
        "formatters": {
            "plain": {
                "format": "%(asctime)s - %(name)s - %(levelname)s - %(message)s",
                "datefmt": "%Y-%m-%d %H:%M:%S",
            },
            "json": {
                "()": JsonFormatter,
                "datefmt": "%Y-%m-%d %H:%M:%S",
            },
        },
        "handlers": handler_config,
        "root": {
            "level": log_level,
            "handlers": handlers,
        },
    }

    logging.config.dictConfig(config)

    if log_level != "DEBUG":
        for noisy in ("apscheduler", "sqlalchemy.engine"):
            logging.getLogger(noisy).setLevel(logging.WARNING)

    structlog.configure(
        processors=[
            structlog.stdlib.filter_by_level,
            structlog.stdlib.add_log_level,
            structlog.stdlib.add_logger_name,
            structlog.processors.TimeStamper(fmt="iso"),
            structlog.stdlib.render_to_log_kwargs,
        ],
        logger_factory=structlog.stdlib.LoggerFactory(),
        wrapper_class=structlog.stdlib.BoundLogger,
        cache_logger_on_first_use=True,
    )
