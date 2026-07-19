use embassy_nrf::pwm::{DutyCycle, SimplePwm};

pub struct Servo {
    pwm_servo: SimplePwm<'static>,
}

impl Servo {
    pub fn new(pwm: SimplePwm<'static>) -> Self {
        Self { pwm_servo: pwm }
    }

    pub fn set_angle(&mut self, angle: u16) {
        //  500 µs ->   0°
        // 1500 µs ->  90°
        // 2500 µs -> 180°
        let angle = if angle > 180 { 180 } else { angle };
        let duty = 500 + 2000 * angle as u32 / 180;
        self.pwm_servo.set_duty(0, DutyCycle::inverted(duty as u16));
    }

    pub fn set_duty(&mut self, duty: u16) {
        self.pwm_servo.set_duty(0, DutyCycle::inverted(duty));
    }
}
