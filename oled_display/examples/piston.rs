#![no_std]
#![no_main]

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
use embassy_ssd1306_physics::Piston;

bind_interrupts!( struct Irqs {
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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

    let mut piston = Piston::new(64, 10, 20, 40);
    {
        let mut gfx = Graphics::new(&mut oled);

        // Position au sommet (fermé)
        piston.set_pos(0);
        piston.draw(&mut gfx, 0.0, true, |_| 0.0, |_| 0.0);

        // Position au milieu
        piston.set_pos(20);
        piston.draw(&mut gfx, 0.0, true, |_| 0.0, |_| 0.0);
    }
    // Animation simple : va-et-vient
    let max_pos = piston.h - piston.piston_h;
    let mut pos: i32 = 0;
    loop {
        {
            let mut gfx = Graphics::new(&mut oled);
            piston.erase(&mut gfx, 0.0, |_| 0.0, |_| 0.0);
            pos = (pos + 1) % (max_pos * 2);
            let actual_pos = if pos > max_pos {
                max_pos * 2 - pos
            } else {
                pos
            };
            piston.set_pos(actual_pos);
            piston.draw(&mut gfx, 0.0, true, |_| 0.0, |_| 0.0);
        }
        oled.flush().await.unwrap();
    }
}
