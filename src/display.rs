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
    geometry::Point,
    mono_font::MonoTextStyleBuilder,
    text::{Baseline, Text, TextStyleBuilder},
};

use crate::state::{DISPLAY_UPDATE, STATE};

type EpdDriver = Epd7in5<
    ExclusiveDevice<Spim<'static>, Output<'static>, embedded_hal_bus::spi::NoDelay>,
    Input<'static>,
    Output<'static>,
    Output<'static>,
    Delay,
>;

#[embassy_executor::task]
pub async fn display_task(
    mut screen: EpdDriver,
    mut spi: ExclusiveDevice<Spim<'static>, Output<'static>, embedded_hal_bus::spi::NoDelay>,
    mut delay: Delay,
) {
    DISPLAY_UPDATE.wait().await;

    let message = {
        let state = STATE.lock().await;

        state.message
    };

    let mut display = Display7in5::default();
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Color::White)
        .background_color(Color::Black)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ =
        Text::with_text_style(message, Point::new(10, 10), style, text_style).draw(&mut display);

    screen
        .update_and_display_frame(&mut spi, &display.buffer(), &mut delay)
        .expect("to work");

    loop {
        embassy_time::Timer::after_secs(10).await;
    }
}
