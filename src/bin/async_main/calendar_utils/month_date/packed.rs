use core::ops::{Add, Range, Sub};

use chrono::{Datelike, Month, Months, NaiveDate, Weekday};
use num_traits::FromPrimitive;

/// CE era month
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonthDate(u16);

impl MonthDate {
    pub const fn new(year: u16, month: Month) -> Self {
        debug_assert!(year < (1 << 12));
        Self((year << 4) | (month.number_from_month() as u16))
    }

    pub const fn year(self) -> u16 {
        self.0 >> 4
    }

    pub const fn month(self) -> Month {
        let month_num = (self.0 & 0b1111) as u8;
        match month_num {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => unreachable!(),
        }
    }

    pub const fn to_start_day_naive(self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year() as i32, self.month().number_from_month(), 1).unwrap()
    }

    pub fn new_from_date(date: NaiveDate) -> Self {
        let (_, year) = date.year_ce();
        let month = Month::from_u8(date.month() as u8).unwrap();

        Self::new(year as u16, month)
    }
}

impl Add<Months> for MonthDate {
    type Output = Self;

    fn add(self, rhs: Months) -> Self::Output {
        let rhs = rhs.as_u32() as u8;
        let lhs = (self.0 & 0b1111) as u8 - 1;

        let sum_months = lhs + rhs;

        let add_years = sum_months / 12;
        let months_remainder = sum_months % 12;

        MonthDate::new(
            self.year() + add_years as u16,
            Month::from_u8(months_remainder + 1).unwrap(),
        )
    }
}

impl Sub<Months> for MonthDate {
    type Output = Self;

    fn sub(self, rhs: Months) -> Self::Output {
        let rhs = rhs.as_u32() as u8;
        let lhs = (self.0 & 0b1111) as u8 - 1;

        if rhs > lhs {
            let sub_years = ((rhs - lhs) / 12) + 1;
            MonthDate::new(
                self.year() - sub_years as u16,
                Month::from_u8(lhs + 1 + (sub_years * 12) - rhs).unwrap(),
            )
        } else {
            MonthDate::new(self.year(), Month::from_u8(lhs - rhs + 1).unwrap())
        }
    }
}
