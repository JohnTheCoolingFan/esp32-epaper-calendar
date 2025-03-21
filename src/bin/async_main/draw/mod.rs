use alloc::string::ToString;

use chrono::DateTime;
use chrono_tz::Tz;
use embedded_graphics::{
    prelude::{DrawTarget, Point, Primitive, Size},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Text, TextStyle},
    Drawable,
};
use weact_studio_epd::TriColor;

use crate::calendar_utils::{all_weekdays_short_en, CalendarMonth};

mod text_styles;
use text_styles::*;

const WEEKDAY_TEXT_STYLE_BLACK: StyleType = STYLE_BLACK_9;
const WEEKDAY_TEXT_STYLE_RED: StyleType = STYLE_RED_9;
const GRID_DAY_STYLE_BLACK: StyleType = STYLE_BLACK_12;
const GRID_DAY_STYLE_RED: StyleType = STYLE_RED_12;

pub async fn draw_calendar<D: DrawTarget<Color = TriColor>>(
    time: &DateTime<Tz>,
    display: &mut D,
) -> Result<(), D::Error> {
    let column_spacing = Point::new(1, 0) + GRID_DAY_STYLE_BLACK.font.character_size.x_axis() * 3;
    let row_spacing = GRID_DAY_STYLE_BLACK.font.character_size.y_axis();

    let local_date_naive = time.naive_local().date();

    let calendar = CalendarMonth::from_date(local_date_naive);
    let today = calendar.today_day_num(local_date_naive);
    let days_grid_anchor = Point::new(14, 26);
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
        .draw(display);
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
        .draw(display);

        if day == today {
            let _ = Rectangle::with_center(
                pos + Point::new(-1, -4),
                Size {
                    width: column_spacing.x as u32,
                    height: GRID_DAY_STYLE_RED.font.character_size.height,
                },
            )
            .into_styled(HIGHLIGHT_STYLE)
            .draw(display);
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
    .draw(display);
    let _ = Text::with_text_style(
        &year,
        year_pos,
        STYLE_RED_18,
        TextStyle::with_alignment(Alignment::Left),
    )
    .draw(display);

    todo!()
}
