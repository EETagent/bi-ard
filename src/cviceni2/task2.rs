#![no_std]
#![no_main]

use arduino_hal::hal::port::PB3;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::prelude::*;
use panic_halt as _;

const DOT_DURATION: u16 = 200;
const DASH_DURATION: u16 = DOT_DURATION * 3;
const SYMBOL_SPACE: u16 = DOT_DURATION;
const LETTER_SPACE: u16 = DOT_DURATION * 3;
const WORD_SPACE: u16 = DOT_DURATION * 7;

const MORSE_ALPHABET: &[(&str, &str)] = &[
    ("A", ".-"),
    ("B", "-..."),
    ("C", "-.-."),
    ("D", "-.."),
    ("E", "."),
    ("F", "..-."),
    ("G", "--."),
    ("H", "...."),
    ("I", ".."),
    ("J", ".---"),
    ("K", "-.-"),
    ("L", ".-.."),
    ("M", "--"),
    ("N", "-."),
    ("O", "---"),
    ("P", ".--."),
    ("Q", "--.-"),
    ("R", ".-."),
    ("S", "..."),
    ("T", "-"),
    ("U", "..-"),
    ("V", "...-"),
    ("W", ".--"),
    ("X", "-..-"),
    ("Y", "-.--"),
    ("Z", "--.."),
];

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let mut led = pins.d11.into_output();

    loop {
        if let Some(b) = nb::block!(serial.read()).ok() {
            let c = b as char;

            if c == ' ' {
                arduino_hal::delay_ms(WORD_SPACE);
            } else {
                if let Some((_, code)) = MORSE_ALPHABET
                    .iter()
                    .find(|(letter, _)| letter.chars().next().unwrap() == c.to_ascii_uppercase())
                {
                    flash_sequence(&mut led, code);
                    arduino_hal::delay_ms(LETTER_SPACE);
                }
            }
        }
    }
}

fn flash_sequence(led: &mut Pin<Output, PB3>, sequence: &str) {
    for symbol in sequence.chars() {
        match symbol {
            '.' => flash_dot(led),
            '-' => flash_dash(led),
            _ => {}
        }
        arduino_hal::delay_ms(SYMBOL_SPACE);
    }
}

fn flash_dot(led: &mut Pin<Output, PB3>) {
    led.set_high();
    arduino_hal::delay_ms(DOT_DURATION);
    led.set_low();
}

fn flash_dash(led: &mut Pin<Output, PB3>) {
    led.set_high();
    arduino_hal::delay_ms(DASH_DURATION);
    led.set_low();
}
