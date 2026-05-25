use embedded_graphics::prelude::DrawTarget;
use weact_studio_epd::TriColor;

use crate::http_apis::weather::ForecastSummary;

pub fn draw_forecast<D: DrawTarget<Color = TriColor>>(
    forecast: ForecastSummary,
) -> Result<(), D::Error> {
    todo!()
}
