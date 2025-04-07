//! A bunch of utils for working with calendar stuff

use core::ops::{Add, Range, Sub};

use chrono::{Datelike, Month, Months, NaiveDate, Weekday};
use num_traits::FromPrimitive;

pub const fn weekday_short_name(val: Weekday) -> &'static str {
    all_weekdays_short_en()[val.num_days_from_monday() as usize]
}

pub const fn all_weekdays() -> [Weekday; 7] {
    [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ]
}

pub const fn all_weekdays_short_en() -> [&'static str; 7] {
    ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
}

pub struct DaysIter {
    range: Range<u8>,
    days_off_mask: DaysOffMask,
}

impl DaysIter {
    const fn new(calendar: &CalendarMonth) -> Self {
        Self {
            range: 0..calendar.days_amount(),
            days_off_mask: calendar.days_off_mask,
        }
    }
}

impl Iterator for DaysIter {
    type Item = (u8, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.range.next()?;
        let is_day_off = self.days_off_mask.is_day0_off(idx);
        let res = (idx, is_day_off);
        Some(res)
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaysOffMask(u32);

impl DaysOffMask {
    pub const fn new(val: u32) -> Self {
        Self(val)
    }

    pub const fn truncate(self, days: u8) -> Self {
        Self(self.0 & ((1_u32 << days) - 1))
    }

    const fn default_days_off(starts_on: Weekday) -> Self {
        let offset = starts_on as u8;
        let init_mask = if offset == 6 {
            0b1000001_u32
        } else {
            0b1100000_u32 >> offset
        };
        Self(
            init_mask
                | (init_mask << 7)
                | (init_mask << (7 * 2))
                | (init_mask << (7 * 3))
                | (init_mask << (7 * 4)),
        )
    }

    pub const fn is_day0_off(self, day: u8) -> bool {
        ((self.0 >> day) & 0b1) != 0
    }

    pub const fn is_day1_off(self, day: u8) -> bool {
        self.is_day0_off(day + 1)
    }
}

/// Data used to describe a calendar month
#[derive(Debug, Clone, Copy)]
pub struct CalendarMonth {
    date: MonthDate,
    days_off_mask: DaysOffMask,
}

impl CalendarMonth {
    /// By default uses the days off mask of saturday and sunday always being days off
    pub fn from_date(date: NaiveDate) -> Self {
        // Assume year is in CE
        let (_, year) = date.year_ce();
        let month = date.month();
        let date = date.with_day0(0).unwrap();
        let weekday = date.weekday();
        Self {
            days_off_mask: DaysOffMask::default_days_off(weekday),
            date: MonthDate::new(year as u16, Month::try_from(month as u8).unwrap()),
        }
    }

    #[inline(always)]
    pub fn month_date(self) -> MonthDate {
        self.date
    }

    pub const fn new_raw(date: MonthDate, day_off_mask: DaysOffMask) -> Self {
        Self {
            date,
            days_off_mask: day_off_mask,
        }
    }

    pub fn days_iter(&self) -> DaysIter {
        DaysIter::new(self)
    }

    pub const fn start_date(&self) -> NaiveDate {
        self.date.to_start_day_naive()
    }

    /// Get teh amount of days in this month
    pub const fn days_amount(&self) -> u8 {
        let start = self.start_date();
        let end = start.checked_add_months(Months::new(1)).unwrap();
        let result = end.signed_duration_since(start).num_days();
        result as u8
    }

    /// Get the day of teh week this month starts on
    pub fn start_weekday(&self) -> Weekday {
        self.start_date().weekday()
    }

    /// Get the number of the week this month starts on
    pub fn start_week_num(&self) -> u8 {
        self.start_date().iso_week().week0() as u8
    }

    /// Get the month this month is from
    pub const fn month(&self) -> Month {
        self.date.month()
    }

    /// Get the year this month is from, clamped to CE and up to 4095
    pub const fn year(&self) -> u16 {
        self.date.year()
    }

    /// Set the days off
    pub const fn set_days_off(&mut self, days_off_mask: DaysOffMask) {
        self.days_off_mask = days_off_mask.truncate(self.days_amount());
    }
}
