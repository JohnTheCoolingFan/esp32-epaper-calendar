use core::ops::Range;

use chrono::{Datelike, Month, Months, NaiveDate, Weekday};

use super::{DaysOffMask, MonthDate};

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
    pub fn month(&self) -> Month {
        self.date.month()
    }

    /// Get the year this month is from, clamped to CE and up to 4095
    pub fn year(&self) -> u16 {
        self.date.year()
    }

    /// Set the days off
    pub const fn set_days_off(&mut self, days_off_mask: DaysOffMask) {
        self.days_off_mask = days_off_mask.truncate(self.days_amount());
    }
}
