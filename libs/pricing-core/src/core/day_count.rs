//! Day count conventions for time calculations.

use chrono::{Datelike, NaiveDate};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

/// Day count conventions for calculating the year fraction between dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayCountConvention {
    /// Actual/360 - Actual days divided by 360.
    /// Commonly used for money market instruments.
    Act360,

    /// Actual/365 (Fixed) - Actual days divided by 365.
    /// Commonly used for sterling and some other markets.
    Act365Fixed,

    /// Actual/Actual (ISDA) - Actual days divided by actual days in year.
    /// Used for bonds and many derivatives.
    ActAct,

    /// 30/360 (Bond Basis) - 30-day months, 360-day years.
    /// Commonly used for corporate bonds.
    Thirty360,

    /// 30E/360 (Eurobond Basis) - European 30/360 variant.
    ThirtyE360,

    /// 30E/360 (ISDA) - ISDA 30/360 variant.
    ThirtyE360Isda,

    /// Actual/365L - Actual days divided by 365 or 366 in leap years.
    Act365L,

    /// NL/365 - Actual days excluding Feb 29, divided by 365.
    NL365,
}

impl DayCountConvention {
    /// Get the name of the convention.
    pub fn name(&self) -> &'static str {
        match self {
            DayCountConvention::Act360 => "ACT/360",
            DayCountConvention::Act365Fixed => "ACT/365 (Fixed)",
            DayCountConvention::ActAct => "ACT/ACT",
            DayCountConvention::Thirty360 => "30/360",
            DayCountConvention::ThirtyE360 => "30E/360",
            DayCountConvention::ThirtyE360Isda => "30E/360 (ISDA)",
            DayCountConvention::Act365L => "ACT/365L",
            DayCountConvention::NL365 => "NL/365",
        }
    }

    /// Calculate the year fraction between two dates.
    ///
    /// # Arguments
    ///
    /// * `start` - The start date.
    /// * `end` - The end date.
    ///
    /// # Returns
    ///
    /// The year fraction as a Decimal. Returns 0 if start >= end.
    pub fn year_fraction(&self, start: NaiveDate, end: NaiveDate) -> Decimal {
        if end <= start {
            return Decimal::ZERO;
        }

        match self {
            DayCountConvention::Act360 => {
                let days = (end - start).num_days();
                Decimal::from(days) / Decimal::from(360)
            }
            DayCountConvention::Act365Fixed => {
                let days = (end - start).num_days();
                Decimal::from(days) / Decimal::from(365)
            }
            DayCountConvention::ActAct => {
                let days = (end - start).num_days();
                // Simplified: use actual days in the year of the start date
                let year_days = if is_leap_year(start.year()) { 366 } else { 365 };
                Decimal::from(days) / Decimal::from(year_days)
            }
            DayCountConvention::Act365L => {
                let days = (end - start).num_days();
                // Use 366 if Feb 29 is in the period, else 365
                let year_days = if contains_feb29(start, end) { 366 } else { 365 };
                Decimal::from(days) / Decimal::from(year_days)
            }
            DayCountConvention::NL365 => {
                let mut days = (end - start).num_days();
                // Subtract 1 if Feb 29 is in the period
                if contains_feb29(start, end) {
                    days -= 1;
                }
                Decimal::from(days) / Decimal::from(365)
            }
            DayCountConvention::Thirty360 => {
                let (d1, m1, y1) = (start.day(), start.month(), start.year());
                let (d2, m2, y2) = (end.day(), end.month(), end.year());

                let d1_adj = if d1 == 31 { 30 } else { d1 };
                let d2_adj = if d2 == 31 && d1_adj == 30 { 30 } else { d2 };

                let num = 360 * (y2 - y1)
                    + 30 * (m2 as i32 - m1 as i32)
                    + (d2_adj as i32 - d1_adj as i32);
                Decimal::from(num) / Decimal::from(360)
            }
            DayCountConvention::ThirtyE360 => {
                let (d1, m1, y1) = (start.day(), start.month(), start.year());
                let (d2, m2, y2) = (end.day(), end.month(), end.year());

                let d1_adj = if d1 == 31 { 30 } else { d1 };
                let d2_adj = if d2 == 31 { 30 } else { d2 };

                let num = 360 * (y2 - y1)
                    + 30 * (m2 as i32 - m1 as i32)
                    + (d2_adj as i32 - d1_adj as i32);
                Decimal::from(num) / Decimal::from(360)
            }
            DayCountConvention::ThirtyE360Isda => {
                let (mut d1, m1, y1) = (start.day(), start.month(), start.year());
                let (mut d2, m2, y2) = (end.day(), end.month(), end.year());

                // Adjust for end of month
                if is_last_day_of_month(start) {
                    d1 = 30;
                }
                if is_last_day_of_month(end) && d1 == 30 {
                    d2 = 30;
                }

                let num = 360 * (y2 - y1) + 30 * (m2 as i32 - m1 as i32) + (d2 as i32 - d1 as i32);
                Decimal::from(num) / Decimal::from(360)
            }
        }
    }

    /// Calculate the number of days between two dates according to this convention.
    pub fn day_count(&self, start: NaiveDate, end: NaiveDate) -> i64 {
        if end <= start {
            return 0;
        }

        match self {
            DayCountConvention::Act360
            | DayCountConvention::Act365Fixed
            | DayCountConvention::ActAct
            | DayCountConvention::Act365L => (end - start).num_days(),
            DayCountConvention::NL365 => {
                let days = (end - start).num_days();
                if contains_feb29(start, end) {
                    days - 1
                } else {
                    days
                }
            }
            DayCountConvention::Thirty360 | DayCountConvention::ThirtyE360 => {
                let yf = self.year_fraction(start, end);
                (yf * Decimal::from(360)).round().to_i64().unwrap_or(0)
            }
            DayCountConvention::ThirtyE360Isda => {
                let yf = self.year_fraction(start, end);
                (yf * Decimal::from(360)).round().to_i64().unwrap_or(0)
            }
        }
    }
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Check if a date is the last day of the month.
fn is_last_day_of_month(date: NaiveDate) -> bool {
    let next_day = date + chrono::Duration::days(1);
    next_day.month() != date.month()
}

/// Check if the period contains February 29.
fn contains_feb29(start: NaiveDate, end: NaiveDate) -> bool {
    // Check each year in the period
    for year in start.year()..=end.year() {
        if !is_leap_year(year) {
            continue;
        }
        let feb29 = NaiveDate::from_ymd_opt(year, 2, 29).unwrap();
        if feb29 > start && feb29 <= end {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn test_act360() {
        let start = date(2024, 1, 1);
        let end = date(2024, 7, 1); // 182 days
        let yf = DayCountConvention::Act360.year_fraction(start, end);
        assert_eq!(yf, dec!(182) / dec!(360));
    }

    #[test]
    fn test_act365_fixed() {
        let start = date(2024, 1, 1);
        let end = date(2024, 7, 1); // 182 days
        let yf = DayCountConvention::Act365Fixed.year_fraction(start, end);
        assert_eq!(yf, dec!(182) / dec!(365));
    }

    #[test]
    fn test_30_360() {
        let start = date(2024, 1, 31);
        let end = date(2024, 7, 31);
        let yf = DayCountConvention::Thirty360.year_fraction(start, end);
        // (360*0 + 30*6 + 0) / 360 = 180/360 = 0.5
        assert_eq!(yf, dec!(0.5));
    }

    #[test]
    fn test_act_act_leap_year() {
        let start = date(2024, 1, 1);
        let end = date(2024, 7, 1);
        let yf = DayCountConvention::ActAct.year_fraction(start, end);
        // 182 / 366 (2024 is a leap year)
        assert_eq!(yf, dec!(182) / dec!(366));
    }

    #[test]
    fn test_day_count() {
        let start = date(2024, 1, 1);
        let end = date(2024, 7, 1);
        let dc = DayCountConvention::Act360.day_count(start, end);
        assert_eq!(dc, 182);
    }

    #[test]
    fn test_contains_feb29() {
        // 2024 is a leap year
        assert!(contains_feb29(date(2024, 1, 1), date(2024, 12, 31)));
        assert!(!contains_feb29(date(2024, 3, 1), date(2024, 12, 31)));
        assert!(!contains_feb29(date(2023, 1, 1), date(2023, 12, 31)));
    }

    #[test]
    fn test_same_date() {
        let d = date(2024, 6, 15);
        let yf = DayCountConvention::Act360.year_fraction(d, d);
        assert_eq!(yf, Decimal::ZERO);
    }

    #[test]
    fn test_reverse_dates() {
        let start = date(2024, 7, 1);
        let end = date(2024, 1, 1);
        let yf = DayCountConvention::Act360.year_fraction(start, end);
        assert_eq!(yf, Decimal::ZERO);
    }
}
