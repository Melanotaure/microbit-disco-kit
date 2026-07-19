#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_time::Timer;
use panic_halt as _;

use embassy_executor::Spawner;
use smart_leds::{RGB8, SmartLedsWrite, hsv::Hsv, hsv::hsv2rgb};
use static_cell::StaticCell;

pub mod led_stripe;
use crate::led_stripe::Ws2812;

static BUFFER_CELL: StaticCell<[u16; 1440]> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Init");
    let p = embassy_nrf::init(Default::default());

    let buffer: &'static mut [u16; 1440] = BUFFER_CELL.init([0; 1440]);
    let mut led_strip: Ws2812<1440> = Ws2812::new(p.PWM0, p.P0_02, buffer);

    // Prepare a buffer for the 60 LEDs
    let mut colors = [RGB8::new(0, 0, 0); 60];

    info!("Let's display a rainbow!");
    for (i, color) in colors.iter_mut().enumerate() {
        let hue = ((i * 255) / 60) as u8;
        let hsv = Hsv {
            hue,
            sat: 255, // Max saturation for pure colors
            val: 128, // Brightness (val)
        };

        // hsv2rgb does the conversion in RGB8
        *color = hsv2rgb(hsv);
    }
    // Non blocking flush via DMA
    led_strip.write(colors.iter().cloned()).unwrap();

    info!("... and rotate it forever...");
    loop {
        colors.rotate_right(2);
        led_strip.write(colors.iter().cloned()).unwrap();
        Timer::after_millis(100).await;
    }
}
