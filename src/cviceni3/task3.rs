#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::{hal::port::{PB1, PB2, PB3, PD7}, port::mode::Output};
use panic_halt as _;

#[derive(PartialEq, Clone, Copy)]
enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    None,
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut led_r = pins.d9.into_output();
    let mut led_g = pins.d11.into_output();
    let mut led_b = pins.d10.into_output();

    let mut buzzer = pins.d7.into_output();

    let button_red    = pins.d2.into_pull_up_input();
    let button_green  = pins.d3.into_pull_up_input();
    let button_blue   = pins.d4.into_pull_up_input();
    let button_yellow = pins.d5.into_pull_up_input();

    const GAME_ROUNDS: u8 = 5;
    const MIN_DISPLAY_TIME: u16 = 800;
    const MAX_DISPLAY_TIME: u16 = 1500;
    const WIN_THRESHOLD: u8 = GAME_ROUNDS / 2 + 1;

    fn set_led_color(
        color: &Color,
        led_r: &mut arduino_hal::port::Pin<Output, PB1>,
        led_g: &mut arduino_hal::port::Pin<Output, PB3>,
        led_b: &mut arduino_hal::port::Pin<Output, PB2>
    ) {
        match color {
            Color::Red => {
                led_r.set_high();
                led_g.set_low();
                led_b.set_low();
            },
            Color::Green => {
                led_r.set_low();
                led_g.set_high();
                led_b.set_low();
            },
            Color::Blue => {
                led_r.set_low();
                led_g.set_low();
                led_b.set_high();
            },
            Color::Yellow => {
                led_r.set_high();
                led_g.set_high();
                led_b.set_low();
            },
            Color::None => {
                led_r.set_low();
                led_g.set_low();
                led_b.set_low();
            },
        }
    }

    fn play_success_tone(buzzer: &mut arduino_hal::port::Pin<Output, PD7>) {
        for _ in 0..100 {
            buzzer.set_high();
            arduino_hal::delay_us(500);
            buzzer.set_low();
            arduino_hal::delay_us(500);
        }
    }

    fn play_failure_tone(buzzer: &mut arduino_hal::port::Pin<Output, PD7>) {
        for _ in 0..150 {
            buzzer.set_high();
            arduino_hal::delay_us(2000);
            buzzer.set_low();
            arduino_hal::delay_us(2000);
        }
    }


    fn rand(seed: &mut u32) -> u32 {
        *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        *seed
    }

    let mut seed: u32 = 0xDEADBEEF;

    let mut wins: u8 = 0;
    let mut cheating: u8 = 0;

    for round in 0..GAME_ROUNDS {
        let r = rand(&mut seed);
        let color = match r % 4 {
            0 => Color::Red,
            1 => Color::Green,
            2 => Color::Blue,
            3 => Color::Yellow,
            _ => Color::None,
        };

        set_led_color(&color, &mut led_r, &mut led_g, &mut led_b);

        let delay_range = (MAX_DISPLAY_TIME - MIN_DISPLAY_TIME + 1) as u32;
        let display_time = MIN_DISPLAY_TIME + (rand(&mut seed) % delay_range) as u16;

        let mut correct_pressed = false;
        let mut elapsed: u16 = 0;
        while elapsed < display_time {
            if (button_red.is_low() &&  button_green.is_low() && button_blue.is_low() && button_yellow.is_low()) {
                correct_pressed = true;
                cheating += 1;
                ufmt::uwriteln!(&mut serial, "p {}", cheating).unwrap();
                break;
            } else if button_red.is_low() {
                if color == Color::Red {
                    correct_pressed = true;
                }
                break;
            } else if button_green.is_low() {
                if color == Color::Green {
                    correct_pressed = true;
                }
                break;
            } else if button_blue.is_low() {
                if color == Color::Blue {
                    correct_pressed = true;
                }
                break;
            } else if button_yellow.is_low() {
                if color == Color::Yellow {
                    correct_pressed = true;
                }
                break;
            }
            arduino_hal::delay_ms(1);
            elapsed += 1;
        }

        set_led_color(&Color::None, &mut led_r, &mut led_g, &mut led_b);

        if correct_pressed {
            play_success_tone(&mut buzzer);
            wins += 1;
            let _ = serial.write_str("Round ");
            ufmt::uwriteln!(&mut serial, "s {}", round + 1).unwrap();
        } else {
            play_failure_tone(&mut buzzer);
            let _ = serial.write_str("Round ");
            ufmt::uwriteln!(&mut serial, "f {}", round + 1).unwrap();
        }

        arduino_hal::delay_ms(1000);
    }

    if wins >= WIN_THRESHOLD {
        ufmt::uwriteln!(&mut serial, "vyhra {}", wins).unwrap();
        for _ in 0..5 {
            led_g.toggle();
            arduino_hal::delay_ms(300);
        }
    } else {
        ufmt::uwriteln!(&mut serial, "prohra {}", wins).unwrap();
        for _ in 0..5 {
            led_r.toggle();
            arduino_hal::delay_ms(300);
        }
    }

    loop {
        arduino_hal::delay_ms(1000);
    }
}
