#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use panic_halt as _;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    bind_interrupts,
    peripherals::TWISPI0,
    temp::Temp,
    twim::{self, Twim},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Delay, Timer};
use hd44780_controller::{
    controller::{
        Controller,
        config::{InitialConfig, RuntimeConfig},
    },
    device::pcf8574::PCF8574Device,
    lcd_println,
};

static SIGNAL: Signal<ThreadModeRawMutex, u16> = Signal::new();

bind_interrupts!( struct Irqs {
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
    TEMP => embassy_nrf::temp::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start...");
    let p = embassy_nrf::init(Default::default());

    let temp = Temp::new(p.TEMP, Irqs);

    let i2c = Twim::new(
        p.TWISPI0,
        Irqs,
        p.P1_00,
        p.P0_26,
        Default::default(),
        &mut [],
    );

    let t_task = temp_task(temp);
    let d_task = display_task(i2c);

    join(t_task, d_task).await;
}

async fn temp_task(mut temp: Temp<'static>) {
    const INTERVAL_MS: u64 = 2000;

    loop {
        let value = temp.read().await.to_num::<u16>();
        info!("{} °C", value);
        SIGNAL.signal(value);
        Timer::after_millis(INTERVAL_MS).await;
    }
}

async fn display_task(i2c: Twim<'static>) {
    let device = PCF8574Device::new(i2c, 0x27, Delay);
    let mut controller = Controller::<PCF8574Device<Twim<'_>, Delay>>::new_async(
        device,
        InitialConfig::default(),
        RuntimeConfig::default(),
    )
    .init()
    .await
    .unwrap();

    let mut col = 0;
    let mut dir: i8 = 1;
    let mut value = 0;
    loop {
        controller.clear().await.unwrap();
        controller.set_cursor_position(0, col).await.unwrap();
        controller.write_str("Hello World!".chars()).await.unwrap();
        col = col.checked_add_signed(dir).unwrap();
        value = match SIGNAL.try_take() {
            Some(v) => v,
            None => value,
        };
        lcd_println!(controller, line = 1, ">temp:{value}")
            .await
            .unwrap();
        Timer::after_millis(500).await;
        if col == 4 {
            dir = -1;
        } else if col == 0 {
            dir = 1;
        }
    }
}
