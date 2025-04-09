//! A bunch of utils for working with calendar stuff

pub mod calendar;
pub mod daysoff_mask;
mod month_date;
use core::ops::{Add, Range, Sub};

pub use calendar::CalendarMonth;
use chrono::{Datelike, Month, Months, NaiveDate, Weekday};
pub use daysoff_mask::DaysOffMask;
pub use month_date::MonthDate;
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
