use alloc::format;

use embedded_graphics::{
    Drawable,
    prelude::{DrawTarget, Point, Size},
    primitives::Rectangle,
    text::{Text, TextStyle},
};
use weact_studio_epd::TriColor;

use crate::{
    draw::text_styles::STYLE_BLACK_7,
    http_apis::weather::{ForecastSummary, ForecastSummaryPeriod},
};

pub fn draw_forecast<D: DrawTarget<Color = TriColor>>(
    forecast: ForecastSummary,
    display: &mut D,
) -> Result<(), D::Error> {
    let anchor = Point::new((228 - SMALL_ICON_SIZE / 2) as i32, 60);
    let offset = Point::new(SMALL_ICON_SIZE as i32 + 8, 0);
    draw_forecast_period_at(forecast.day, anchor, display)?;
    draw_forecast_period_at(forecast.evening, anchor + offset, display)?;
    draw_forecast_period_at(forecast.morning, anchor - offset, display)?;
    Ok(())
}

fn text_black_7_at<D: DrawTarget<Color = TriColor>>(
    text: &str,
    at: Point,
    display: &mut D,
) -> Result<(), D::Error> {
    Text::with_text_style(text, at, STYLE_BLACK_7, TextStyle::default()).draw(display)?;
    Ok(())
}

fn draw_forecast_period_at<D: DrawTarget<Color = TriColor>>(
    forecast_period: ForecastSummaryPeriod,
    pos: Point,
    display: &mut D,
) -> Result<(), D::Error> {
    draw_small_icon_at(
        pos,
        super::icons::weather_icons::icon_id_to_icon(forecast_period.weather_code_max),
        display,
    )?;

    let temp_anchor = pos + Point::new(4, SMALL_ICON_SIZE as i32 + 8);

    text_black_7_at(
        &format!("{:+.0}", forecast_period.apparent_temperature_range.1),
        temp_anchor,
        display,
    )?;
    text_black_7_at(
        &format!("{:+.0}", forecast_period.apparent_temperature_range.0),
        temp_anchor + Point::new(0, 8),
        display,
    )?;
    text_black_7_at("°C", temp_anchor + Point::new(16, 4), display)?;

    let wind_anchor = pos + Point::new(0, SMALL_ICON_SIZE as i32 + 28);

    text_black_7_at(
        &format!("{:.0} m/s", forecast_period.wind_speed_avg),
        wind_anchor,
        display,
    )?;

    Ok(())
}

const SMALL_ICON_SIZE: u32 = 32;

fn draw_small_icon_at<D: DrawTarget<Color = TriColor>>(
    position: Point,
    icon_data: &[u8],
    display: &mut D,
) -> Result<(), D::Error> {
    display.fill_contiguous(
        &Rectangle::new(position, Size::new(SMALL_ICON_SIZE, SMALL_ICON_SIZE)),
        icon_data.iter().flat_map(|byte| {
            (0..8).map(|i| {
                if *byte & (1 << i) == 0 {
                    TriColor::White
                } else {
                    TriColor::Black
                }
            })
        }),
    )
}
