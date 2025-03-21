use chrono::DateTime;
use chrono_tz::Tz;
use embedded_graphics::prelude::DrawTarget;

use crate::calendar_utils::CalendarMonth;

pub async fn draw_calendar<D: DrawTarget>(
    time: &DateTime<Tz>,
    display: &mut D,
) -> Result<(), D::Error> {
    let calendar = CalendarMonth::from_date(time.naive_local().date());
    todo!()
}
