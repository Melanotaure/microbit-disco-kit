#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Level, Output, OutputDrive, Pull};
use embassy_time::{Delay, Timer};
use tm1637::TM1637;

use {defmt_rtt as _, panic_probe as _};

const DIGITS: [u8; 16] = [
    0x3f, 0x06, 0x5b, 0x4f, //
    0x66, 0x6d, 0x7d, 0x07, //
    0x7f, 0x6f, 0x77, 0x7c, //
    0x39, 0x5e, 0x79, 0x71, //
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut clk = Output::new(p.P0_17, Level::Low, OutputDrive::Standard);
    let mut dio = Flex::new(p.P0_01);
    dio.set_as_input_output(Pull::None, OutputDrive::Standard0HighDrive1);
    let mut del = Delay;

    let mut tm = TM1637::new(&mut clk, &mut dio, &mut del);

    tm.init().expect("Init error");
    tm.clear().expect("Clear error");

    let mut d0: u8;
    let mut d1: u8;
    let mut dot = false;

    loop {
        info!("Standard display");
        tm.print_hex(0, &[0, 1, 2, 3]).unwrap();
        Timer::after_secs(2).await;
        tm.clear().expect("Clear error");

        info!("Progressive raw display with brightness.");
        for i in 0..255 {
            d0 = DIGITS[i & 0x0F];
            d1 = DIGITS[(i + 1) & 0x0F];
            if dot == true {
                d1 &= !0x80;
            } else {
                d1 |= 0x80
            }
            dot = !dot;

            tm.print_raw(0, &[d0, d1]).expect("ph error");

            tm.print_raw(3, &[i as u8]).expect("pr error");

            tm.set_brightness(i as u8 >> 5).expect("bright error");
            Timer::after_millis(300).await;
        }
    }
}
