from sqlalchemy import Column, DateTime, Integer, Numeric, String

from ingestion.db.session import Base


class EquityPrice(Base):
    __tablename__ = "equity_prices"

    time = Column(DateTime(timezone=True), primary_key=True)
    symbol = Column(String(16), primary_key=True)
    open = Column(Numeric(19, 4))
    high = Column(Numeric(19, 4))
    low = Column(Numeric(19, 4))
    close = Column(Numeric(19, 4))
    volume = Column(Integer)


class OptionPrice(Base):
    __tablename__ = "options_data"

    time = Column(DateTime(timezone=True), primary_key=True)
    symbol = Column(String(16), primary_key=True)
    expiry = Column(DateTime(timezone=True), primary_key=True)
    strike = Column(Numeric(19, 4), primary_key=True)
    option_type = Column(String(4), primary_key=True)
    bid = Column(Numeric(19, 4))
    ask = Column(Numeric(19, 4))
    iv = Column(Numeric(19, 6))
    delta = Column(Numeric(19, 6))
    gamma = Column(Numeric(19, 6))
    theta = Column(Numeric(19, 6))
    vega = Column(Numeric(19, 6))
    volume = Column(Integer)
    open_interest = Column(Integer)


class YieldCurvePoint(Base):
    __tablename__ = "yield_curve"

    curve_date = Column(DateTime(timezone=True), primary_key=True)
    tenor = Column(String(16), primary_key=True)
    rate = Column(Numeric(19, 6))
    source = Column(String(32))


class CpiData(Base):
    __tablename__ = "cpi_data"

    release_date = Column(DateTime(timezone=True), primary_key=True)
    series_id = Column(String(16), primary_key=True)
    value = Column(Numeric(19, 6))
    period = Column(String(8))
