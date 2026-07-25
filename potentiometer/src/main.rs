#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    saadc::{ChannelConfig, Config, InterruptHandler, Saadc},
};
use embassy_time::Timer;

use potentiometer::led_matrix::{display_matrix, display_value};

use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SAADC => InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let config = Config::default();
    let chan_cfg = ChannelConfig::single_ended(p.P0_02);
    let mut saadc = Saadc::new(p.SAADC, Irqs, config, [chan_cfg]);

    let rows = [
        Output::new(p.P0_21, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_22, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_15, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_19, Level::Low, OutputDrive::Standard),
    ];

    let cols = [
        Output::new(p.P0_28, Level::High, OutputDrive::Standard),
        Output::new(p.P0_11, Level::High, OutputDrive::Standard),
        Output::new(p.P0_31, Level::High, OutputDrive::Standard),
        Output::new(p.P1_05, Level::High, OutputDrive::Standard),
        Output::new(p.P0_30, Level::High, OutputDrive::Standard),
    ];

    spawner.spawn(display_matrix(rows, cols).expect("display_matrix error"));

    let mut buffer = [0; 1];
    loop {
        saadc.sample(&mut buffer).await;
        let analog_value = buffer[0];
        let safe_value = analog_value.max(0) as u16;
        display_value(safe_value).await;
        Timer::after_millis(500).await;
    }
}
