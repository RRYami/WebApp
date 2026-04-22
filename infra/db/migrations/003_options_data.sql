CREATE TABLE IF NOT EXISTS options_data (
    time TIMESTAMPTZ NOT NULL,
    symbol TEXT NOT NULL,
    expiry TIMESTAMPTZ NOT NULL,
    strike NUMERIC(19,4) NOT NULL,
    option_type TEXT NOT NULL,
    bid NUMERIC(19,4),
    ask NUMERIC(19,4),
    iv NUMERIC(19,6),
    delta NUMERIC(19,6),
    gamma NUMERIC(19,6),
    theta NUMERIC(19,6),
    vega NUMERIC(19,6),
    volume BIGINT,
    open_interest BIGINT,
    PRIMARY KEY (time, symbol, expiry, strike, option_type)
);

SELECT create_hypertable('options_data', 'time', if_not_exists => TRUE);

CREATE INDEX idx_options_symbol ON options_data (symbol, time DESC);
CREATE INDEX idx_options_expiry ON options_data (expiry, time DESC);
