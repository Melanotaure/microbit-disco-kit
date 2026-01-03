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
    Peri, bind_interrupts,
    gpio::{AnyPin, Input, Level, Output, OutputDrive, Pull},
    peripherals::{self, TWISPI0},
    temp::Temp,
    twim::{self, Twim},
    uarte,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel, signal::Signal};
use embassy_time::{Delay, Instant, Timer};
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
static DISTANCE_SIGNAL: Signal<ThreadModeRawMutex, f32> = Signal::new();

bind_interrupts!( struct Irqs {
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
    TEMP => embassy_nrf::temp::InterruptHandler;
    UARTE0 => uarte::InterruptHandler<peripherals::UARTE0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start...");
    let p = embassy_nrf::init(Default::default());

    // UART Configuration
    let mut config = uarte::Config::default();
    config.baudrate = uarte::Baudrate::BAUD115200;

    let uart = uarte::Uarte::new(p.UARTE0, p.P0_08, p.P0_06, Irqs, config);
    spawner.spawn(uart_task(uart)).unwrap();

    // HCSR04 Sensor
    spawner
        .spawn(hcsr04_task(p.P0_14.into(), p.P0_03.into()))
        .unwrap();

    // Temp Sensor
    let temp = Temp::new(p.TEMP, Irqs);

    let i2c = Twim::new(
        p.TWISPI0,
        Irqs,
        p.P1_00,
        p.P0_26,
        Default::default(),
        &mut [],
    );

    // Spawn tasks
    let t_task = temp_task(temp);
    let d_task = display_task(i2c);

    join(t_task, d_task).await;
}

async fn temp_task(mut temp: Temp<'static>) {
    info!("Temp task started...");
    const INTERVAL_MS: u64 = 1000;

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
    info!("Display task started...");
    let device = PCF8574Device::new(i2c, 0x27, Delay);
    let mut controller = Controller::<PCF8574Device<Twim<'_>, Delay>>::new_async(
        device,
        InitialConfig::default(),
        RuntimeConfig::default(),
    )
    .init()
    .await
    .unwrap();

    controller.clear().await.unwrap();

    let mut value = 0;
    let mut distance = 0.0;
    loop {
        value = match SIGNAL.try_take() {
            Some(v) => v,
            None => value,
        };
        lcd_println!(controller, line = 0, "temp: {value} ")
            .await
            .unwrap();

        distance = match DISTANCE_SIGNAL.try_take() {
            Some(v) => v,
            None => distance,
        };
        lcd_println!(controller, line = 1, "dist: {:.2}  ", distance)
            .await
            .unwrap();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn uart_task(mut uart: uarte::Uarte<'static>) {
    info!("UART task started...");
    loop {
        // The task wait for a message from the channel
        let msg = UART_CHANNEL.receive().await;

        // We look for the end of the message in case we use a fixed buffer
        // or we send the fixed buffer. Here we just send the whole string:
        let _ = uart.write(msg.as_bytes()).await;
    }
}

#[embassy_executor::task]
async fn hcsr04_task(trig_pin: Peri<'static, AnyPin>, echo_pin: Peri<'static, AnyPin>) {
    info!("HCSR04 task started...");
    let mut trig = Output::new(trig_pin, Level::Low, OutputDrive::Standard);
    let mut echo = Input::new(echo_pin, Pull::None);

    let mut temp = 22.0;

    loop {
        // Trigger the sensor
        trig.set_high();
        Timer::after_micros(10).await;
        trig.set_low();

        // Wait for echo start
        echo.wait_for_rising_edge().await;
        let start = Instant::now();

        // Wait for echo end
        echo.wait_for_falling_edge().await;
        let end = Instant::now();

        // Calculate distance in cm
        let duration = end.duration_since(start);
        let microseconds = duration.as_micros();
        temp = match SIGNAL.try_take() {
            Some(v) => v as f32,
            None => temp,
        };
        let speed_of_sound_m_s = 331.3 + (0.606 * temp);
        let distance_cm = (microseconds as f32 * speed_of_sound_m_s) / 20000.0;

        info!("Distance: {} cm", distance_cm);
        let mut s: String<64> = String::new();
        if write!(s, ">Dist:{}\n", distance_cm).is_ok() {
            UART_CHANNEL.send(s).await
        }
        DISTANCE_SIGNAL.signal(distance_cm);
        Timer::after_millis(500).await;
    }
}
