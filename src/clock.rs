use embedded_graphics::geometry::Point;
use epd_waveshare::epd7in5_v2::Display7in5;

use crate::display::{Font, STATUS_BAR_HEIGHT, render_message};

pub fn draw_clock(time: u32, display: &mut Display7in5) {
    render_message(
        display,
        "14:56",
        Point::new(260, (STATUS_BAR_HEIGHT + 158) as i32),
        Some(Font::Logisoso),
    );
}
