use embedded_graphics::{
    Drawable,
    geometry::{Point, Size},
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
};
use epd_waveshare::{color::Color, epd7in5_v2::Display7in5};
use heapless::{String, Vec};

use crate::{
    display::{Font, STATUS_BAR_HEIGHT, render_message},
    state::{DISPLAY_UPDATE, STATE},
};

pub async fn publish_debug_message(message: &str) {
    {
        let mut state = STATE.lock().await;

        if state.messages.is_full() {
            state.messages.remove(0);
        }

        let mut line = String::<64>::new();

        if line.push_str(message).is_ok() {
            let _ = state.messages.push(line);
        }

        DISPLAY_UPDATE.signal(());
    }
}

pub fn draw_debug_window(messages: Vec<String<64>, 5>, display: &mut Display7in5) {
    let height: u32 = 90;
    let width: u32 = 200;
    let offset_top: i32 = (STATUS_BAR_HEIGHT + 4) as i32;
    let offset_left: i32 = 600;

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::White)
        .stroke_width(1)
        .build();

    let _ = Rectangle::new(
        Point::new(offset_left, offset_top as i32),
        Size::new(width, height),
    )
    .into_styled(style)
    .draw(display);

    for (i, message) in messages.iter().enumerate() {
        render_message(
            display,
            &message,
            Point::new(offset_left + 8, offset_top + 8 + (14 * i) as i32),
            Some(Font::Regular),
        );
    }
}
