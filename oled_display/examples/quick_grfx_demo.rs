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
use embassy_ssd1306_graphics::{Graphics, bezier_quad, circle, ellipse, fill_triangle, line};

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

    loop {
        oled.clear();

        {
            let mut gfx = Graphics::new(&mut oled);

            // Primitives de ce crate
            line(&mut gfx, 0, 0, 127, 63, true);
            circle(&mut gfx, 64, 32, 20, true);
            ellipse(&mut gfx, 64, 32, 40, 16, true);
            fill_triangle(&mut gfx, 64, 4, 20, 59, 108, 59, true);
            bezier_quad(&mut gfx, 10, 50, 64, 5, 118, 50, 24, true);
        }
        // ↑ borrow libéré oled à nouveau accessible

        // Texte et flush via le driver directement
        oled.draw_str(40, 7, b"Hello!");
        oled.flush().await.unwrap();
    }
}
