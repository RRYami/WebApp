CREATE TABLE IF NOT EXISTS cpi_data (
    release_date TIMESTAMPTZ NOT NULL,
    series_id TEXT NOT NULL,
    value NUMERIC(19,6),
    period TEXT,
    PRIMARY KEY (release_date, series_id)
);

SELECT create_hypertable('cpi_data', 'release_date', if_not_exists => TRUE);

CREATE INDEX idx_cpi_series ON cpi_data (series_id, release_date DESC);
