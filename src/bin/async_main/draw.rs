use chrono::DateTime;
use chrono_tz::Tz;
use embedded_graphics::prelude::DrawTarget;

pub async fn draw_calendar<D: DrawTarget>(
    time: &DateTime<Tz>,
    display: &mut D,
) -> Result<(), D::Error> {
    todo!()
}
