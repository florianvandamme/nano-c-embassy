#![no_std]
#![no_main]

mod display;
mod message;
mod state;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals,
    spim::{self, Config, Spim},
};

use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use epd_waveshare::{epd7in5_v2::Epd7in5, prelude::WaveshareDisplay};
use panic_probe as _;

use crate::display::display_task;
use crate::message::publish_message_task;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    bind_interrupts!(struct Irqs {
        TWISPI0 => spim::InterruptHandler<peripherals::TWISPI0>;
    });

    let busy = Input::new(p.P0_23, Pull::None); // BUSY (P0.23) - D7
    let cs = Output::new(p.P1_02, Level::High, OutputDrive::Standard); // CS (P1.02) - D10
    let dc = Output::new(p.P0_27, Level::Low, OutputDrive::Standard); // DC (P0.27) - D9
    let rst = Output::new(p.P0_21, Level::Low, OutputDrive::Standard); // RST (P0.21) - D8
    let pwr = Output::new(p.P1_14, Level::High, OutputDrive::Standard); // PWR (P1.14) - D6

    let mut config = Config::default();
    config.frequency = spim::Frequency::M4;
    config.mode = spim::MODE_0;
    config.bit_order = spim::BitOrder::MsbFirst;

    let spim = Spim::new_txonly(p.TWISPI0, Irqs, p.P0_13, p.P1_01, config);
    let mut spi = ExclusiveDevice::new_no_delay(spim, cs).unwrap();
    let mut delay = Delay;

    let epd7in5 = Epd7in5::new(&mut spi, busy, dc, rst, &mut delay, None).expect("to work");

    spawner.spawn(display_task(epd7in5, spi, delay, pwr).unwrap());
    spawner.spawn(publish_message_task().unwrap());

    core::future::pending::<()>().await;
}
