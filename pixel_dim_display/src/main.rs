#![no_std]
#![no_main]

mod animations;
use animations::*;

use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_nrf::pwm::{DutyCycle, Prescaler, SimplePwm};
use embassy_nrf::{
    gpio::{Level, Output, OutputDrive},
    pwm::SimpleConfig,
};
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start of LED Matrix PWM app");
    let p = embassy_nrf::init(Default::default());

    // --- 1. Lines configuration (Anodes) ---
    // They are set to Low so that no LED is lit at startup
    let rows: [Output<'static>; 5] = [
        Output::new(p.P0_21, Level::Low, OutputDrive::Standard), // Row 1
        Output::new(p.P0_22, Level::Low, OutputDrive::Standard), // Row 2
        Output::new(p.P0_15, Level::Low, OutputDrive::Standard), // Row 3
        Output::new(p.P0_24, Level::Low, OutputDrive::Standard), // Row 4
        Output::new(p.P0_19, Level::Low, OutputDrive::Standard), // Row 5
    ];

    // --- 2. PWM0 configuration (Columns 1 to 4) ---
    // "top" is defined at 255 to have the scale 0-255
    let pwm0 = SimplePwm::new_4ch(
        p.PWM0,
        p.P0_28, // Col 1
        p.P0_11, // Col 2
        p.P0_31, // Col 3
        p.P1_05, // Col 4
        &SimpleConfig::default(),
    );
    pwm0.set_prescaler(Prescaler::Div1); // 16MHz / 1 = Very high frequency to avoid flickering
    pwm0.set_max_duty(255);

    // --- 3. PWM1 configuration (Column 5) ---
    let pwm1 = SimplePwm::new_1ch(
        p.PWM1,
        p.P0_30, // Col 5
        &SimpleConfig::default(),
    );
    pwm1.set_prescaler(Prescaler::Div1);
    pwm1.set_max_duty(255);

    // --- 4. Display task launch ---
    spawner.spawn(led_matrix_task(pwm0, pwm1, rows)).unwrap();
    spawner.spawn(animation_task()).unwrap();

    // Your code can continue here asynchronously
    loop {
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn led_matrix_task(
    mut pwm0: SimplePwm<'static>,
    mut pwm1: SimplePwm<'static>,
    mut rows: [Output<'static>; 5],
) {
    let mut current_row = 0;
    let mut frame_buffer: [[u8; 5]; 5] = [[0; 5]; 5];

    loop {
        // 1. Light off the previous line (Ghosting prevention)
        for r in &mut rows {
            r.set_low();
        }

        if let Some(new_frame) = FRAME_SIGNAL.try_take() {
            frame_buffer = new_frame;
        }
        // 2. Get the data of the current line
        let row_data = frame_buffer[current_row];

        // 3. Apply the intensities on the two PWM instances
        pwm0.set_duty(0, DutyCycle::normal(row_data[0] as u16));
        pwm0.set_duty(1, DutyCycle::normal(row_data[1] as u16));
        pwm0.set_duty(2, DutyCycle::normal(row_data[2] as u16));
        pwm0.set_duty(3, DutyCycle::normal(row_data[3] as u16));

        pwm1.set_duty(0, DutyCycle::normal(row_data[4] as u16)); // 5th column on the 2nd PWM

        // 4. Light up the current line (Anode)
        rows[current_row].set_high();

        // 5. Wait before moving to the next line
        // 2ms per line = 10ms for a full refresh (100Hz)
        Timer::after_millis(2).await;

        current_row = (current_row + 1) % 5;
    }
}

#[embassy_executor::task]
async fn animation_task() {
    loop {
        rotating_bar_animation().await;
        Timer::after_millis(500).await;

        moving_triangle_animation().await;
        Timer::after_millis(500).await;

        dimmed_line_animation().await;
        Timer::after_millis(500).await;

        beatin_heart_animation().await;
        Timer::after_millis(500).await;
    }
}
