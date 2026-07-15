#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_ssd1306_graphics::Graphics;
use embassy_ssd1306_physics::{Gear, GearPair};
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

    let pair = GearPair::new(
        Gear::new(30, 32, 12, 8, 4, 70, 3), // moteur (cx/cy réels)
        Gear::new(0, 0, 18, 12, 4, 70, 5),  // récepteur (cx/cy calculés)
        0.0,                                // g2 à droite de g1
        trig::cos,
        trig::sin,
    );

    let mut angle: f32 = 0.0;
    loop {
        {
            let mut gfx = Graphics::new(&mut oled);
            pair.erase(&mut gfx, angle, trig::cos, trig::sin);
            angle += 0.05;
            pair.draw(&mut gfx, angle, true, trig::cos, trig::sin);
        }
        oled.flush().await.unwrap();
    }
}
