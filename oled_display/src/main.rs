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
        oled.draw_str(0, 0, b"Hello World!");
        oled.flush().await.unwrap();
    }
}
