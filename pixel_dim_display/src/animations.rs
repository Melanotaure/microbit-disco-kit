use core::f32::consts::PI;
use defmt::*;
use defmt_rtt as _;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use libm::sinf; // When no_std, use the crate libm for the math functions

// A signal that contains the 5x5 buffer
pub static FRAME_SIGNAL: Signal<ThreadModeRawMutex, [[u8; 5]; 5]> = Signal::new();

// Rotating bar animation
const ROTATING_BAR: [[[u8; 5]; 5]; 4] = [
    [
        [1, 0, 1, 0, 1],
        [1, 1, 0, 0, 1],
        [1, 0, 1, 0, 1],
        [1, 0, 0, 1, 1],
        [1, 0, 1, 0, 1],
    ],
    [
        [1, 0, 1, 0, 1],
        [1, 0, 1, 0, 1],
        [1, 0, 1, 0, 1],
        [1, 0, 1, 0, 1],
        [1, 0, 1, 0, 1],
    ],
    [
        [1, 0, 1, 0, 1],
        [1, 0, 0, 1, 1],
        [1, 0, 1, 0, 1],
        [1, 1, 0, 0, 1],
        [1, 0, 1, 0, 1],
    ],
    [
        [1, 0, 1, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 1, 0, 1],
    ],
];

// Triangle shape definition (1 = LED on, 0 = LED off)
const TRIANGLE: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 1],
    [0, 0, 0, 1, 1],
    [0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
];

// Heart shape definition (1 = LED on, 0 = LED off)
const HEART: [[u8; 5]; 5] = [
    [0, 1, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [1, 1, 1, 1, 1],
    [0, 1, 1, 1, 0],
    [0, 0, 1, 0, 0],
];

const DEFAULT_BRIGHTNESS: u8 = 127; // 50% brightness

pub async fn rotating_bar_animation() {
    // Rotating bar animation (10 cycles)
    info!("Rotating bar animation");
    for _ in 0..10 {
        for i in 0..4 {
            Timer::after_millis(100).await;
            let mut new_frame = [[0u8; 5]; 5];
            for r in 0..5 {
                for c in 0..5 {
                    new_frame[r][c] = ROTATING_BAR[i][r][c] * DEFAULT_BRIGHTNESS;
                }
            }
            FRAME_SIGNAL.signal(new_frame);
        }
    }
}

pub async fn moving_triangle_animation() {
    // Moving triangle animation
    info!("Moving triangle animation");
    let mut new_frame = [[0u8; 5]; 5];
    for r in 0..5 {
        for c in 0..5 {
            new_frame[r][c] = TRIANGLE[r][c] * DEFAULT_BRIGHTNESS;
        }
    }
    FRAME_SIGNAL.signal(new_frame);

    for _ in 0..20 {
        Timer::after_millis(100).await;
        let tmp = new_frame[0];
        new_frame[0] = new_frame[1];
        new_frame[1] = new_frame[2];
        new_frame[2] = new_frame[3];
        new_frame[3] = new_frame[4];
        new_frame[4] = tmp;
        FRAME_SIGNAL.signal(new_frame);
    }
}

pub async fn dimmed_line_animation() {
    // Dimmed line animation
    info!("Dimmed line animation");
    const BLOCK_SIZE: usize = 5;
    const MIN_BRIGHTNESS: u8 = 0x0F;
    const STEPS: usize = BLOCK_SIZE - 1;
    const RUNS: usize = 4;
    let mut dimmed: u8 = 0;
    let mut inc = false;
    let mut new_frame = [[0u8; 5]; 5];
    FRAME_SIGNAL.signal(new_frame);

    for i in 0..(1 + 3 * RUNS * STEPS) {
        Timer::after_millis(100).await;

        for column in 0..BLOCK_SIZE {
            for row in 0..BLOCK_SIZE {
                let diff;
                let mut step = (0xFF - MIN_BRIGHTNESS) / (STEPS as u8);

                /*
                 * Depending on the iteration, use different
                 * pixel values for:
                 * - vertical lines
                 */
                if i < 1 * RUNS * STEPS {
                    diff = if dimmed > column as u8 {
                        dimmed - (column as u8)
                    } else {
                        column as u8 - dimmed
                    };
                /*
                 * - horizontal lines
                 */
                } else if i < 2 * RUNS * STEPS {
                    diff = if dimmed > row as u8 {
                        dimmed - (row as u8)
                    } else {
                        row as u8 - dimmed
                    };

                    /*
                     * - diagonal lines
                     */
                } else {
                    let dist = column + row;

                    diff = if 2 * dimmed > dist as u8 {
                        2 * dimmed - (dist as u8)
                    } else {
                        (dist as u8) - 2 * dimmed
                    };
                    step /= 2;
                }

                new_frame[row][column] = MIN_BRIGHTNESS + diff * step;
                FRAME_SIGNAL.signal(new_frame);
            }
        }

        if dimmed == 0 || dimmed == STEPS as u8 {
            inc = !inc;
        }
        if inc {
            dimmed += 1;
        } else {
            dimmed -= 1;
        }
    }
}

pub async fn beatin_heart_animation() {
    let mut step: f32 = 0.0;
    // Beating heart animation
    info!("Beating heart animation");
    for _ in 0..500 {
        // Beating heart animation based on sine wave
        // Global intensity computing (0.0 à 1.0)
        // The function sin(step) oscillates between -1 and 1.
        // By doing (sin + 1) / 2, we oscillate between 0 and 1.
        let brightness = (sinf(step) + 1.0) / 2.0;

        // Let's apply a simple gamma correction curve (x^2)
        // for a more natural brightness sensation
        let corrected_intensity = (brightness * brightness * 255.0) as u8;

        // Let's create a new 5x5 buffer with this intensity everywhere
        let mut new_frame = [[0u8; 5]; 5];
        for r in 0..5 {
            for c in 0..5 {
                new_frame[r][c] = corrected_intensity * HEART[r][c];
            }
        }

        // Send the new buffer to the display task via a Signal
        FRAME_SIGNAL.signal(new_frame);

        // Pulse speed
        step += 0.05;
        if step > 2.0 * PI {
            step -= 2.0 * PI;
        }
        // Let's define a refresh rate of the logic at 50Hz (20ms)
        Timer::after_millis(20).await;
    }
}
