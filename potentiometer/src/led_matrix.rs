use embassy_nrf::gpio::Output;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::Timer;

use crate::character_table::{EMPTY_CHAR, FONT_ASCII};

// A signal that contains the 5x5 buffer
pub static DISPLAY_SIGNAL: Signal<ThreadModeRawMutex, [[u8; 5]; 5]> = Signal::new();

#[embassy_executor::task]
pub async fn display_matrix(mut rows: [Output<'static>; 5], mut cols: [Output<'static>; 5]) {
    let mut frame_buffer: [[u8; 5]; 5] = [[0; 5]; 5];

    loop {
        if let Some(new_frame) = DISPLAY_SIGNAL.try_take() {
            frame_buffer = new_frame;
        }

        for (row_idx, row_data) in frame_buffer.iter().enumerate() {
            // 1. Switch off all the rows to prevent ghosting
            for row in rows.iter_mut() {
                row.set_low();
            }

            // 2. Configure the columns for the current line
            for (col_idx, &pixel) in row_data.iter().enumerate() {
                if pixel == 1 {
                    cols[col_idx].set_low(); // Activer la cathode
                } else {
                    cols[col_idx].set_high(); // Désactiver la cathode
                }
            }

            // 3. Switch on the current line
            rows[row_idx].set_high();

            // 4. Hold the state for the multiplexing (i.e.: 2ms per line)
            Timer::after_millis(2).await;
        }
    }
}

pub async fn scroll_text(text: &str) {
    let mut current_matrix = &EMPTY_CHAR;

    for c in text.chars() {
        let next_matrix = get_char_matrix(c);

        for offset in 0..=5 {
            let frame = shift_left(&current_matrix, &next_matrix, offset);

            DISPLAY_SIGNAL.signal(frame);
            Timer::after_millis(150).await;
        }

        current_matrix = next_matrix;
    }

    for offset in 0..=5 {
        let frame = shift_left(&current_matrix, &[[0; 5]; 5], offset);
        DISPLAY_SIGNAL.signal(frame);
        Timer::after_millis(150).await;
    }
}

fn shift_left(current: &[[u8; 5]; 5], next: &[[u8; 5]; 5], offset: usize) -> [[u8; 5]; 5] {
    let mut frame = [[0; 5]; 5];

    for row in 0..5 {
        for col in 0..5 {
            if col + offset < 5 {
                // Left part: current character out
                frame[row][col] = current[row][col + offset];
            } else if col + offset == 5 {
                // Empty column between two characters
                frame[row][col] = 0;
            } else {
                // Right part: new character in
                frame[row][col] = next[row][col + offset - 6]; // -6 pour compenser l'espace
            }
        }
    }
    frame
}

pub fn get_char_matrix(c: char) -> &'static [[u8; 5]; 5] {
    if c.is_ascii_digit() {
        &FONT_ASCII[c as usize - 0x30]
    } else {
        &EMPTY_CHAR
    }
}

pub async fn display_value(value: u16) {
    let mut buffer = itoa::Buffer::new();
    let text_value = buffer.format(value);

    scroll_text(text_value).await;
}
