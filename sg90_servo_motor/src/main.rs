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
    // Initialisation des périphériques du nRF52833
    let p = embassy_nrf::init(Default::default());

    // 1. Configuration de la PWM
    let mut pwm_config = SimpleConfig::default();

    // Le système tourne à 16 MHz. Un diviseur de 16 nous donne une base
    // de temps de 1 MHz. Ainsi, 1 "tick" matériel = 1 microseconde (µs).
    pwm_config.prescaler = Prescaler::Div16;

    // Pour une période de 20 ms, il nous faut 20 000 ticks de 1 µs.
    pwm_config.max_duty = 20_000;

    // 2. Déploiement du driver
    // On instancie un PWM sur un seul canal. Le "Pad 0" du micro:bit v2
    // correspond physiquement à la broche P0.02.
    // L'API exige la référence à la configuration (&pwm_config).
    let pwm = SimplePwm::new_1ch(p.PWM0, p.P0_02, &pwm_config);
    let mut servo = Servo::new(pwm);

    loop {
        // 3. Pilotage du servo (Canal 0)

        // Position 0° : impulsion de 1 ms = 500 µs
        servo.set_angle(0);
        // servo.set_duty(500);
        Timer::after_millis(1000).await;

        // Position 90° (centre) : impulsion de 1.5 ms = 1500 µs
        servo.set_angle(90);
        // servo.set_duty(1500);
        Timer::after_millis(1000).await;

        // Position 180° : impulsion de 2 ms = 2500 µs
        servo.set_angle(180);
        // servo.set_duty(2500);
        Timer::after_millis(1000).await;
    }
}
