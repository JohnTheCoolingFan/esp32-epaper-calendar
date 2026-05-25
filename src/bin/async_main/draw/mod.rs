use alloc::string::ToString;

use chrono::{DateTime, Datelike};
use chrono_tz::Tz;
use embedded_graphics::{
    Drawable,
    prelude::{DrawTarget, Point, Primitive, Size},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text, TextStyle},
};
use weact_studio_epd::TriColor;

use crate::calendar_utils::{CalendarMonth, all_weekdays_short_en};

pub mod calendar;
mod text_styles;
#[cfg(feature = "weather")]
pub mod weather;
