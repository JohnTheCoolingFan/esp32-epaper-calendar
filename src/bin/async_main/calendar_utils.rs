//! A bunch of utils for working with calendar stuff

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

/// Data used to describe a calendar month
#[derive(Debug, Clone, Copy)]
pub struct CalendarMonth {
    day_off_mask: u32,
    start_date: NaiveDate,
}

impl CalendarMonth {
    fn from_date(date: NaiveDate) -> Self {
        let date = date.with_day0(0).unwrap();
        let weekday = date.weekday();
        Self {
            day_off_mask: Self::default_days_off(weekday),
            start_date: date,
        }
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
        self.start_date
    }

    pub fn days_amount(&self) -> u8 {
        let start = self.start_date;
        let end = start + Months::new(1);
        let result = end.signed_duration_since(start).num_days();
        result.try_into().unwrap()
    }

    pub fn start_weekday(&self) -> Weekday {
        self.start_date.weekday()
    }

    pub fn start_week_num(&self) -> u8 {
        self.start_date.iso_week().week0() as u8
    }

    pub fn month(&self) -> Month {
        Month::from_u32(self.start_date.month()).unwrap()
    }

    pub fn set_days_off(&mut self, day_off_mask: u32) {
        self.day_off_mask = day_off_mask & ((1_u32 << self.days_amount()) - 1)
    }
}
