CREATE TABLE IF NOT EXISTS yield_curve (
    curve_date TIMESTAMPTZ NOT NULL,
    tenor TEXT NOT NULL,
    rate NUMERIC(19,6),
    source TEXT,
    PRIMARY KEY (curve_date, tenor)
);

SELECT create_hypertable('yield_curve', 'curve_date', if_not_exists => TRUE);

CREATE INDEX idx_yield_curve_tenor ON yield_curve (tenor, curve_date DESC);
