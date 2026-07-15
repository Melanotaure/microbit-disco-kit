#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_ssd1306_graphics::Graphics;
use embassy_ssd1306_physics::Compass;
use panic_halt as _;

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    peripherals::TWISPI0,
    twim::{self, Twim},
};
use embassy_ssd1306::Ssd1306;
use embedded_trig_f32 as trig;

bind_interrupts!( struct Irqs {
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Init...");
    let p = embassy_nrf::init(Default::default());

    let i2c = Twim::new(
        p.TWISPI0,
        Irqs,
        p.P1_00,
        p.P0_26,
        Default::default(),
        &mut [],
    );

    let mut oled = Ssd1306::new(i2c, 0x3C);
    oled.init().await.unwrap();
    let mut gfx = Graphics::new(&mut oled);
    let compass = Compass::new(64, 32, 20);
    compass.draw(&mut gfx, 0.785, true, trig::cos, trig::sin); // NE
    oled.flush().await.unwrap();
}
