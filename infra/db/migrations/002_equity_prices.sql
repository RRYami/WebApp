CREATE TABLE IF NOT EXISTS equity_prices (
    time TIMESTAMPTZ NOT NULL,
    symbol TEXT NOT NULL,
    open NUMERIC(19,4),
    high NUMERIC(19,4),
    low NUMERIC(19,4),
    close NUMERIC(19,4),
    volume BIGINT,
    PRIMARY KEY (time, symbol)
);

SELECT create_hypertable('equity_prices', 'time', if_not_exists => TRUE);

CREATE INDEX idx_equity_prices_symbol ON equity_prices (symbol, time DESC);
