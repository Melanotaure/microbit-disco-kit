#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use panic_halt as _;

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    peripherals::TWISPI0,
    twim::{self, Twim},
};
use embassy_ssd1306::Ssd1306;
use embassy_ssd1306_graphics::Graphics;
use embassy_ssd1306_physics::Pendulum;
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

    // Encastrement plus grand : 20×8 au lieu de 16×5
    let pend = Pendulum::with_wall(32, 10, 30, 20, 8);
    {
        let mut gfx = Graphics::new(&mut oled);
        pend.draw(&mut gfx, 0.2, true, trig::cos, trig::sin);
    }

    let mut angle: f32 = 0.0;
    let mut delta = 0.05;
    loop {
        {
            let mut gfx = Graphics::new(&mut oled);
            pend.erase(&mut gfx, angle, trig::cos, trig::sin);
            angle += delta;
            pend.draw(&mut gfx, angle, true, trig::cos, trig::sin);
            if angle > 0.8 {
                delta = -0.05
            } else if angle < -0.8 {
                delta = 0.05
            }
        }
        oled.flush().await.unwrap();
    }
}
