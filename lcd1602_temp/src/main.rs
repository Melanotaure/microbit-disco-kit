#![no_std]
#![no_main]

use core::fmt::Write;
use heapless::String;

use defmt::info;
use defmt_rtt as _; // global logger
use panic_halt as _;

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nrf::{
    bind_interrupts,
    peripherals::{self, TWISPI0},
    temp::Temp,
    twim::{self, Twim},
    uarte,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal};
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
static UART_CHANNEL: Channel<ThreadModeRawMutex, String<64>, 8> = Channel::new();

bind_interrupts!( struct Irqs {
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
    TEMP => embassy_nrf::temp::InterruptHandler;
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start...");
    let p = embassy_nrf::init(Default::default());

    let mut config = uarte::Config::default();
    config.baudrate = uarte::Baudrate::BAUD115200;

    let uart = uarte::Uarte::new(p.UARTE0, p.P0_08, p.P0_06, Irqs, config);
    spawner.spawn(uart_task(uart)).unwrap();

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
        let mut s: String<64> = String::new();
        if write!(s, ">Temp:{}\n", value).is_ok() {
            UART_CHANNEL.send(s).await
        }

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

#[embassy_executor::task]
async fn uart_task(mut uart: uarte::Uarte<'static>) {
    loop {
        // La tâche attend qu'un message arrive dans le canal
        let msg = UART_CHANNEL.receive().await;

        // On cherche la fin réelle du message (au cas où il est plus court que 32 octets)
        // ou on envoie le buffer fixe. Ici on envoie tout pour simplifier :
        let _ = uart.write(msg.as_bytes()).await;
    }
}
