use chrono::Weekday;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DaysOffMask(u32);

impl DaysOffMask {
    pub const fn new(val: u32) -> Self {
        Self(val)
    }

    pub const fn truncate(self, days: u8) -> Self {
        Self(self.0 & ((1_u32 << days) - 1))
    }

    pub(crate) const fn default_days_off(starts_on: Weekday) -> Self {
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
