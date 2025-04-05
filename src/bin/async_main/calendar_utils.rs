//! A bunch of utils for working with calendar stuff

use core::ops::Range;

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
    days_off_mask: u32,
}

impl DaysIter {
    fn new(calendar: &CalendarMonth) -> Self {
        Self {
            range: 0..calendar.days_amount(),
            days_off_mask: calendar.day_off_mask,
        }
    }
}

impl Iterator for DaysIter {
    type Item = (u8, bool);

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.range.next()?;
        let is_day_off = ((self.days_off_mask >> idx) & 0b1) != 0;
        let res = (idx, is_day_off);
        Some(res)
    }
}

/// CE era month
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonthDate(u16);

impl MonthDate {
    pub fn new(year: u16, month: Month) -> Self {
        debug_assert!(year < (1 << 12));
        Self(year << 4 | (month.number_from_month() as u16))
    }

    pub fn year(self) -> u16 {
        self.0 >> 4
    }

    fn month(self) -> Month {
        let month_num = (self.0 & 0b1111) as u8;
        Month::try_from(month_num).unwrap()
    }

    pub fn to_start_day_naive(self) -> NaiveDate {
        NaiveDate::from_ymd(self.year() as i32, self.month().number_from_month(), 1)
    }
}

/// Data used to describe a calendar month
#[derive(Debug, Clone, Copy)]
pub struct CalendarMonth {
    date: MonthDate,
    day_off_mask: u32,
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
            day_off_mask: Self::default_days_off(weekday),
            date: MonthDate::new(year as u16, Month::try_from(month as u8).unwrap()),
        }
    }

    pub fn days_iter(&self) -> DaysIter {
        DaysIter::new(self)
    }

    const fn default_days_off(starts_on: Weekday) -> u32 {
        let offset = starts_on as u8;
        let init_mask = if offset == 6 {
            0b1000001_u32
        } else {
            0b1100000_u32 >> offset
        };
        init_mask
            | (init_mask << 7)
            | (init_mask << (7 * 2))
            | (init_mask << (7 * 3))
            | (init_mask << (7 * 4))
    }

    pub fn start_date(&self) -> NaiveDate {
        self.date.to_start_day_naive()
    }

    /// Get teh amount of days in this month
    pub fn days_amount(&self) -> u8 {
        let start = self.start_date();
        let end = start + Months::new(1);
        let result = end.signed_duration_since(start).num_days();
        result.try_into().unwrap()
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
    pub fn month(&self) -> Month {
        self.date.month()
    }

    /// Get the year this month is from, clamped to CE era and up to 4095
    pub fn year(&self) -> u16 {
        self.date.year()
    }

    /// Set the days off
    pub fn set_days_off(&mut self, day_off_mask: u32) {
        self.day_off_mask = day_off_mask & ((1_u32 << self.days_amount()) - 1)
    }

    /// Get `today` as a number in this month
    pub fn today_day_num(&self, today: NaiveDate) -> u8 {
        (today - self.start_date()).num_days().try_into().unwrap()
    }
}
