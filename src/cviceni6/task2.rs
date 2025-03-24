#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let mut red_led  = pins.d13.into_output();
    let mut green_led = pins.d12.into_output();
    let mut blue_led  = pins.d11.into_output();

    let x_axis = pins.a0.into_analog_input(&mut adc);
    let y_axis = pins.a1.into_analog_input(&mut adc);

    loop {
        let x_value = x_axis.analog_read(&mut adc);
        let y_value = y_axis.analog_read(&mut adc);

        ufmt::uwriteln!(&mut serial, "X: {}, Y: {}", x_value, y_value).unwrap();

        match serial.read() {
            Ok(byte) => {
                match byte {
                    b'R' | b'r' => {
                        red_led.set_high();
                        green_led.set_low();
                        blue_led.set_low();
                    }
                    b'G' | b'g' => {
                        red_led.set_low();
                        green_led.set_high();
                        blue_led.set_low();
                    }
                    b'B' | b'b' => {
                        red_led.set_low();
                        green_led.set_low();
                        blue_led.set_high();
                    }
                    b'O' | b'o' => {
                        red_led.set_low();
                        green_led.set_low();
                        blue_led.set_low();
                    }
                    _ => {
                        // Nic
                    }
                }
            }
            Err(nb::Error::WouldBlock) => {
                // Nic
            }
            Err(_) => { }
        }

        arduino_hal::delay_ms(500);
    }
}
