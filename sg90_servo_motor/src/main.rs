#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_halt as _;

use embassy_executor::Spawner;
use embassy_nrf::pwm::{Prescaler, SimpleConfig, SimplePwm};
use embassy_time::Timer;

pub mod sg90_servo_motor;
use crate::sg90_servo_motor::Servo;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialisation of the nRF52833 peripherics
    let p = embassy_nrf::init(Default::default());

    // PWM Configuration
    let mut pwm_config = SimpleConfig::default();

    // The system runs at 16 MHz. A 16 division gives us a time base
    // of 1 MHz. So, 1 "tick" = 1 microsecond (µs).
    pwm_config.prescaler = Prescaler::Div16;

    // For a period of 20 ms, we need 20 000 ticks of 1 µs.
    pwm_config.max_duty = 20_000;

    // Instanciation of a PWM on only one channel. The "Pad 0" of the micro:bit v2
    // physicly corresponds to the pin P0.02.
    let pwm = SimplePwm::new_1ch(p.PWM0, p.P0_02, &pwm_config);
    let mut servo = Servo::new(pwm);

    loop {
        // Angle 0° : 1 ms = 500 µs pulse
        servo.set_angle(0);
        // servo.set_duty(500);
        Timer::after_millis(1000).await;

        // Angle 90° (center) : 1.5 ms = 1500 µs pulse
        servo.set_angle(90);
        // servo.set_duty(1500);
        Timer::after_millis(1000).await;

        // Angle 180° : 2 ms = 2500 µs pulse
        servo.set_angle(180);
        // servo.set_duty(2500);
        Timer::after_millis(1000).await;
    }
}
