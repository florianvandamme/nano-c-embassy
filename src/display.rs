use embassy_nrf::{
    gpio::{Input, Output},
    spim::Spim,
};
use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{
    color::Color,
    epd7in5_v2::{Display7in5, Epd7in5},
    prelude::WaveshareDisplay,
};

use embedded_graphics::{
    Drawable,
    geometry::{Point, Size},
    mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10},
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text, TextStyleBuilder, renderer::TextRenderer},
};
use heapless::{String, Vec};
use u8g2_fonts::{U8g2TextStyle, fonts::u8g2_font_open_iconic_embedded_2x_t};

use crate::{
    display::Font::IconicEmbedded,
    state::{DISPLAY_UPDATE, STATE},
};

const STATUS_BAR_HEIGHT: u32 = 26;

type EpdDriver = Epd7in5<
    ExclusiveDevice<Spim<'static>, Output<'static>, embedded_hal_bus::spi::NoDelay>,
    Input<'static>,
    Output<'static>,
    Output<'static>,
    Delay,
>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Font {
    #[default]
    Mono6x10,
    IconicEmbedded,
}

pub fn render_message(
    screen: &mut Display7in5,
    message: &str,
    position: Point,
    font: Option<Font>,
) {
    match font.unwrap_or_default() {
        Font::Mono6x10 => {
            let style = MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(Color::White)
                .background_color(Color::Black)
                .build();

            draw_message(screen, message, style, position);
        }
        Font::IconicEmbedded => {
            let style = U8g2TextStyle::new(u8g2_font_open_iconic_embedded_2x_t, Color::White);

            draw_message(screen, message, style, position);
        }
    }
}

fn draw_message<R>(screen: &mut Display7in5, message: &str, style: R, position: Point)
where
    R: TextRenderer<Color = Color>,
{
    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(message, position, style, text_style).draw(screen);
}

#[embassy_executor::task]
pub async fn display_task(
    mut screen: EpdDriver,
    mut spi: ExclusiveDevice<Spim<'static>, Output<'static>, embedded_hal_bus::spi::NoDelay>,
    mut delay: Delay,
    _pwr: Output<'static>,
) {
    loop {
        DISPLAY_UPDATE.wait().await;

        let messages = {
            let state = STATE.lock().await;

            state.messages.clone()
        };

        let mut display = Display7in5::default();

        // @TODO:: Get from state
        draw_status_bar(true, &mut display);
        draw_debug_window(messages, &mut display);

        screen
            .update_and_display_frame(&mut spi, &display.buffer(), &mut delay)
            .expect("to work");
    }
}

fn draw_status_bar(is_connected: bool, display: &mut Display7in5) {
    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Color::White)
        .stroke_width(1)
        .build();

    let _ = Rectangle::new(Point::new(0, 0), Size::new(800, STATUS_BAR_HEIGHT))
        .into_styled(style)
        .draw(display);

    if is_connected {
        render_message(display, &"J", Point::new(760, 4), Some(IconicEmbedded));
    }

    render_message(display, &"I", Point::new(780, 4), Some(IconicEmbedded));
}

fn draw_debug_window(messages: Vec<String<64>, 5>, display: &mut Display7in5) {
    let height: u32 = 66;
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
            Point::new(offset_left + 8, offset_top + 8 + (10 * i) as i32),
            None,
        );
    }
}
