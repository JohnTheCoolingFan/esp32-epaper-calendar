use core::ops::{Add, Range, Sub};

use chrono::{Datelike, Month, Months, NaiveDate, Weekday};
use num_traits::FromPrimitive;

/// CE era month
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonthDate(NaiveDate);

impl MonthDate {
    pub const fn new(year: u16, month: Month) -> Self {
        Self(NaiveDate::from_ymd_opt(year as i32, month.number_from_month(), 1).unwrap())
    }

    pub fn year(self) -> u16 {
        self.0.year_ce().1 as u16
    }

    pub fn month(self) -> Month {
        Month::from_u8(self.0.month() as u8).unwrap()
    }

    pub const fn to_start_day_naive(self) -> NaiveDate {
        self.0
    }

    pub fn new_from_date(date: NaiveDate) -> Self {
        Self(date.with_day(1).unwrap())
    }
}

impl Add<Months> for MonthDate {
    type Output = Self;

    fn add(self, rhs: Months) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Sub<Months> for MonthDate {
    type Output = Self;

    fn sub(self, rhs: Months) -> Self::Output {
        Self(self.0 - rhs)
    }
}
