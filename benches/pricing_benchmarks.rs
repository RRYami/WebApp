//! Benchmarks for pricing calculations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pricing_lib::prelude::*;

fn benchmark_option_pricing(c: &mut Criterion) {
    c.bench_function("black_scholes_call", |b| {
        b.iter(|| {
            let _ = BlackScholes::price_call(
                black_box(dec!(100)),
                black_box(dec!(100)),
                black_box(dec!(0.05)),
                black_box(dec!(0.2)),
                black_box(1.0),
            );
        });
    });

    c.bench_function("black_scholes_greeks", |b| {
        b.iter(|| {
            let _ = BlackScholes::greeks(
                black_box(dec!(100)),
                black_box(dec!(100)),
                black_box(dec!(0.05)),
                black_box(dec!(0.2)),
                black_box(1.0),
                black_box(OptionType::Call),
            );
        });
    });

    c.bench_function("implied_volatility", |b| {
        let price = dec!(10.45);
        b.iter(|| {
            let _ = BlackScholes::implied_volatility(
                black_box(price),
                black_box(dec!(100)),
                black_box(dec!(100)),
                black_box(dec!(0.05)),
                black_box(1.0),
                black_box(OptionType::Call),
                black_box(Some(0.2)),
            );
        });
    });
}

fn benchmark_bond_pricing(c: &mut Criterion) {
    use chrono::NaiveDate;

    c.bench_function("zero_coupon_bond", |b| {
        let bond = ZeroCouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            DayCountConvention::Act360,
        )
        .unwrap();
        let rate = InterestRate::continuous(dec!(0.05));
        let pricing_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        b.iter(|| {
            let _ = bond.price_with_yield(black_box(&rate), black_box(pricing_date));
        });
    });

    c.bench_function("coupon_bond", |b| {
        let bond = CouponBond::new(
            Money::new(dec!(1000), CurrencyCode::USD),
            dec!(0.06),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2029, 1, 1).unwrap(),
            2,
            DayCountConvention::Thirty360,
        )
        .unwrap();
        let rate = InterestRate::continuous(dec!(0.05));
        let pricing_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

        b.iter(|| {
            let _ = bond.price_with_yield(black_box(&rate), black_box(pricing_date));
        });
    });
}

fn benchmark_money_operations(c: &mut Criterion) {
    c.bench_function("money_addition", |b| {
        let m1 = Money::new(dec!(100.50), CurrencyCode::USD);
        let m2 = Money::new(dec!(50.25), CurrencyCode::USD);

        b.iter(|| {
            let _ = m1.checked_add(&m2);
        });
    });

    c.bench_function("money_scalar_mul", |b| {
        let m = Money::new(dec!(100.50), CurrencyCode::USD);
        b.iter(|| {
            let _ = m.mul_scalar(dec!(1.05));
        });
    });
}

criterion_group!(
    benches,
    benchmark_option_pricing,
    benchmark_bond_pricing,
    benchmark_money_operations
);
criterion_main!(benches);
