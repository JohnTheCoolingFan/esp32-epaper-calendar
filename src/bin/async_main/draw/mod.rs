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

mod text_styles;
use text_styles::*;

const WEEKDAY_TEXT_STYLE_BLACK: StyleType = STYLE_BLACK_9;
const WEEKDAY_TEXT_STYLE_RED: StyleType = STYLE_RED_9;
const GRID_DAY_STYLE_BLACK: StyleType = STYLE_BLACK_12;
const GRID_DAY_STYLE_RED: StyleType = STYLE_RED_12;
// mini-calendars in triplet do not print weekday labels
//#[cfg(feature = "calendar-style-triplet")]
//const SMALL_WEEKDAY_TEXT_STYLE_BLACK: StyleType = STYLE_BLACK_7;
//#[cfg(feature = "calendar-style-triplet")]
//const SMALL_WEEKDAY_TEXT_STYLE_RED: StyleType = STYLE_RED_7;
#[cfg(feature = "calendar-style-triplet")]
const SMALL_GRID_DAY_STYLE_BLACK: StyleType = STYLE_BLACK_7;
#[cfg(feature = "calendar-style-triplet")]
const SMALL_GRID_DAY_STYLE_RED: StyleType = STYLE_RED_7;

pub fn draw_calendars<D: DrawTarget<Color = TriColor>>(
    time: &DateTime<Tz>,
    calendar: CalendarMonth,
    #[cfg(feature = "calendar-style-triplet")] calendar_before: CalendarMonth,
    #[cfg(feature = "calendar-style-triplet")] calendar_after: CalendarMonth,
    display: &mut D,
) -> Result<(), D::Error> {
    draw_main_calendar(time, calendar, display)?;
    #[cfg(feature = "calendar-style-triplet")]
    draw_smaller_calendar(false, calendar_before, display)?;
    #[cfg(feature = "calendar-style-triplet")]
    draw_smaller_calendar(true, calendar_after, display)?;
    #[cfg(feature = "calendar-style-bignum")]
    draw_current_day_big(time, calendar, display)?;

    Ok(())
}

fn draw_main_calendar<D: DrawTarget<Color = TriColor>>(
    time: &DateTime<Tz>,
    calendar: CalendarMonth,
    display: &mut D,
) -> Result<(), D::Error> {
    let column_spacing = Point::new(1, 0) + GRID_DAY_STYLE_BLACK.font.character_size.x_axis() * 3;
    let row_spacing = GRID_DAY_STYLE_BLACK.font.character_size.y_axis();

    let local_date_naive = time.naive_local().date();

    let today = local_date_naive.day0() as u8;
    let days_grid_anchor = Point::new(14, 48);
    let weekday_anchor = days_grid_anchor + Point::new(0, -14);

    const HIGHLIGHT_STYLE: PrimitiveStyle<TriColor> = PrimitiveStyleBuilder::new()
        .stroke_color(TriColor::Red)
        .stroke_width(2)
        .build();

    for (i, day_of_week) in all_weekdays_short_en().into_iter().enumerate() {
        let pos = weekday_anchor + column_spacing * (i as i32);
        let style = if i > 4 {
            WEEKDAY_TEXT_STYLE_RED
        } else {
            WEEKDAY_TEXT_STYLE_BLACK
        };
        let _ = Text::with_text_style(
            day_of_week,
            pos,
            style,
            TextStyle::with_alignment(Alignment::Center),
        )
        .draw(display)?;
    }

    let start_offset = calendar.start_weekday().num_days_from_monday() as u8;
    for (day, is_day_off) in calendar.days_iter() {
        let column = (day + start_offset) % 7;
        let row = (day + start_offset) / 7;

        let pos = days_grid_anchor + column_spacing * (column as i32) + row_spacing * row as u32;

        let text = (day + 1).to_string();

        let _ = Text::with_text_style(
            &text,
            pos,
            if is_day_off {
                GRID_DAY_STYLE_RED
            } else {
                GRID_DAY_STYLE_BLACK
            },
            TextStyle::with_alignment(Alignment::Center),
        )
        .draw(display)?;

        if day == today {
            Rectangle::with_center(
                pos + Point::new(-1, -4),
                Size {
                    width: column_spacing.x as u32,
                    height: GRID_DAY_STYLE_RED.font.character_size.height,
                },
            )
            .into_styled(HIGHLIGHT_STYLE)
            .draw(display)?;
        }
    }

    let month = calendar.month();
    let year = calendar.year().to_string();
    let month_name = month.name();

    let month_name_pos = Point::new(4, 19);
    let year_pos = Point::new(116, 19);
    let _ = Text::with_text_style(
        month_name,
        month_name_pos,
        STYLE_BLACK_18,
        TextStyle::with_alignment(Alignment::Left),
    )
    .draw(display)?;
    let _ = Text::with_text_style(
        &year,
        year_pos,
        STYLE_RED_18,
        TextStyle::with_alignment(Alignment::Left),
    )
    .draw(display)?;

    Ok(())
}

#[cfg(feature = "calendar-style-triplet")]
fn draw_smaller_calendar<D: DrawTarget<Color = TriColor>>(
    is_lower: bool,
    calendar: CalendarMonth,
    display: &mut D,
) -> Result<(), D::Error> {
    let mut anchor = Point::new(164, -8);
    if is_lower {
        anchor.y += 62;
    }

    let column_spacing =
        Point::new(0, 0) + SMALL_GRID_DAY_STYLE_BLACK.font.character_size.x_axis() * 3;
    let row_spacing = Point::new(0, -3) + SMALL_GRID_DAY_STYLE_BLACK.font.character_size.y_axis();

    let days_grid_anchor = Point::new(14, 28) + anchor;

    let start_offset = calendar.start_weekday().num_days_from_monday() as u8;
    for (day, is_day_off) in calendar.days_iter() {
        let column = (day + start_offset) % 7;
        let row = (day + start_offset) / 7;

        let pos = days_grid_anchor + column_spacing * (column as i32) + row_spacing * row as i32;

        let text = (day + 1).to_string();

        let _ = Text::with_text_style(
            &text,
            pos,
            if is_day_off {
                SMALL_GRID_DAY_STYLE_RED
            } else {
                SMALL_GRID_DAY_STYLE_BLACK
            },
            TextStyle::with_alignment(Alignment::Center),
        )
        .draw(display)?;
    }

    let month = calendar.month();
    let year = calendar.year().to_string();
    let month_name = month.name();

    let month_name_pos = Point::new(12, 19) + anchor;
    let year_pos = Point::new(80, 19) + anchor;
    let _ = Text::with_text_style(
        month_name,
        month_name_pos,
        STYLE_BLACK_12,
        TextStyle::with_alignment(Alignment::Left),
    )
    .draw(display)?;
    let _ = Text::with_text_style(
        &year,
        year_pos,
        STYLE_RED_12,
        TextStyle::with_alignment(Alignment::Left),
    )
    .draw(display)?;

    Ok(())
}

#[cfg(feature = "calendar-style-bignum")]
fn draw_current_day_big<D: DrawTarget<Color = TriColor>>(
    time: &DateTime<Tz>,
    calendar: CalendarMonth,
    display: &mut D,
) -> Result<(), D::Error> {
    use num_traits::FromPrimitive;

    let anchor = Point::new(228, 50);
    let current_day = time.day() as u8;
    let is_day_off = calendar
        .days_iter()
        .find_map(|(day, is_off)| {
            if (day + 1) == current_day {
                Some(is_off)
            } else {
                None
            }
        })
        .unwrap_or(false);
    let weekday =
        chrono::Weekday::from_u8((calendar.start_weekday() as u8 + current_day - 1) % 7).unwrap();

    Text::with_text_style(
        current_day.to_string().as_str(),
        anchor,
        if is_day_off {
            STYLE_RED_24
        } else {
            STYLE_BLACK_24
        },
        TextStyle::with_alignment(Alignment::Center),
    )
    .draw(display)?;

    Text::with_text_style(
        match weekday {
            chrono::Weekday::Mon => "Monday",
            chrono::Weekday::Tue => "Tuesday",
            chrono::Weekday::Wed => "Wednesday",
            chrono::Weekday::Thu => "Thursday",
            chrono::Weekday::Fri => "Friday",
            chrono::Weekday::Sat => "Saturday",
            chrono::Weekday::Sun => "Sunday",
        },
        anchor + Point::new(0, 24),
        if is_day_off {
            STYLE_RED_18
        } else {
            STYLE_BLACK_18
        },
        TextStyle::with_alignment(Alignment::Center),
    )
    .draw(display)?;

    Ok(())
}
