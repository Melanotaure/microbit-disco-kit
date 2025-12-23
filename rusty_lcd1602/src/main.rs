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
use embassy_time::{Delay, Timer};
use hd44780_controller::{
    controller::{
        Controller,
        config::{InitialConfig, RuntimeConfig},
    },
    device::pcf8574::PCF8574Device,
    lcd_println,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start...");
    let p = embassy_nrf::init(Default::default());

    bind_interrupts!( struct Irqs {
        TWISPI0 => twim::InterruptHandler<TWISPI0>;
    });

    let i2c = Twim::new(
        p.TWISPI0,
        Irqs,
        p.P1_00,
        p.P0_26,
        Default::default(),
        &mut [],
    );
    let device = PCF8574Device::new(i2c, 0x27, Delay);
    let mut controller = Controller::<PCF8574Device<Twim<'_>, Delay>>::new_async(
        device,
        InitialConfig::default(),
        RuntimeConfig::default(),
    )
    .init()
    .await
    .unwrap();

    let mut count = 0u8;
    let mut col = 0;
    let mut dir: i8 = 1;
    loop {
        controller.clear().await.unwrap();
        controller.set_cursor_position(0, col).await.unwrap();
        controller.write_str("Hello World!".chars()).await.unwrap();
        col = col.checked_add_signed(dir).unwrap();
        lcd_println!(controller, line = 1, "count: {count}")
            .await
            .unwrap();
        count = count.wrapping_add(1);
        Timer::after_millis(500).await;
        if col == 4 {
            dir = -1;
            info!("Bump! <-");
        } else if col == 0 {
            dir = 1;
            info!("Bump! ->");
        }
    }
}
