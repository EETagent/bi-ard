#![no_std]
#![no_main]

use ag_lcd::{Cursor, Display, LcdDisplay, Lines};
use embedded_dht_rs::dht11::Dht11;
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use port_expander::dev::pcf8574::Pcf8574;
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let delay_dht = arduino_hal::Delay::new();
    let delay_lcd = arduino_hal::Delay::new();

    let dht_sensor_pin = pins.d2.into_opendrain_high();
    let mut dht11 = Dht11::new(dht_sensor_pin, delay_dht);

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let lm335 = pins.a0.into_analog_input(&mut adc);

    let light = pins.a1.into_analog_input(&mut adc);

    let button1 = pins.d7.into_pull_up_input(); // SW1: Down
    let button2 = pins.d6.into_pull_up_input(); // SW2: Select
    let button3 = pins.d5.into_pull_up_input(); // SW3: Back
    let button4 = pins.d4.into_pull_up_input(); // SW4: Up

    let sda = pins.a4.into_pull_up_input();
    let scl = pins.a5.into_pull_up_input();
    let i2c_bus = arduino_hal::i2c::I2c::new(dp.TWI, sda, scl, 50000);
    let mut i2c_expander = Pcf8574::new(i2c_bus, true, true, true);

    let mut lcd: LcdDisplay<_, _> = LcdDisplay::new_pcf8574(&mut i2c_expander, delay_lcd)
        .with_lines(Lines::TwoLines)
        .with_display(Display::On)
        .with_cursor(Cursor::Off)
        .build();

    let menu_items = ["1. Temperature", "2. LM335", "3. Light"];
    let mut selected_index = 0;
    let items_per_screen = 2;
    let mut current_page = 0;

    update_menu_display(&mut lcd, &menu_items, selected_index, current_page, items_per_screen);

    let mut menu_open = true;
    let mut menu_title_rendered = false;

    let mut temp: u8 = 0;

    loop {
        let mut button_pressed = false;

        if button1.is_low() {
            ufmt::uwriteln!(&mut serial, "Down pressed").unwrap();
            if selected_index < menu_items.len() - 1 {
                selected_index += 1;
                if selected_index >= (current_page + 1) * items_per_screen {
                    current_page += 1;
                }
                button_pressed = true;
            }
        } else if button4.is_low() {
            ufmt::uwriteln!(&mut serial, "Up pressed").unwrap();
            if selected_index > 0 {
                selected_index -= 1;
                if selected_index < current_page * items_per_screen {
                    current_page -= 1;
                }
                button_pressed = true;
            }
        } else if button2.is_low() {
            ufmt::uwriteln!(&mut serial, "Selected: {}", selected_index).unwrap();
            button_pressed = true;
            menu_open = false;
        } else if button3.is_low() {
            ufmt::uwriteln!(&mut serial, "Back to menu").unwrap();
            button_pressed = true;
            menu_open = true;
            menu_title_rendered = false;
            selected_index = 0;
        }

        if button_pressed && menu_open {
            update_menu_display(&mut lcd, &menu_items, selected_index, current_page, items_per_screen);
            arduino_hal::delay_ms(200);
        }

        if !menu_open {
            if !menu_title_rendered {
                lcd.clear();
                lcd.set_position(0, 0);
                lcd.print(menu_items[selected_index]);
                menu_title_rendered = true;
            }

            lcd.set_position(0, 1);
            if selected_index == 0 {
                let temp_str = convert_to_string::<2>(temp);
                lcd.print(temp_str.as_str());
                ufmt::uwriteln!(&mut serial, "temp {}", temp).unwrap();
            } else if selected_index == 1 {
                let lm335_value = lm335.analog_read(&mut adc);

                let temp_str = convert_to_string::<3>(lm335_value);
                lcd.print(temp_str.as_str());
                ufmt::uwriteln!(&mut serial, "temp {}", lm335_value).unwrap();
            } else if selected_index == 2 {
                let light_value = light.analog_read(&mut adc);

                let temp_str = convert_to_string::<3>(light_value);
                lcd.print(temp_str.as_str());

                ufmt::uwriteln!(&mut serial, "light {}", light_value).unwrap();
            }
        }

        match dht11.read() {
                Ok(sensor_reading) => {
                    // let hum = sensor_reading.humidity;
                    temp = sensor_reading.temperature;
                }
                Err(_error) => {}
        }


        arduino_hal::delay_ms(1000);
    }
}

fn update_menu_display<T, D>(
    lcd: &mut LcdDisplay<T, D>,
    menu_items: &[&str],
    selected_index: usize,
    current_page: usize,
    items_per_screen: usize,
) where
    T: OutputPin + Sized,
    D: DelayNs + Sized,
{
    lcd.clear();

    let start_index = current_page * items_per_screen;

    for i in 0..items_per_screen {
        let menu_index = start_index + i;
        if menu_index < menu_items.len() {
            lcd.set_position(0, i as u8);
            if menu_index == selected_index {
                lcd.print("> ");
            } else {
                lcd.print("  ");
            }

            lcd.print(menu_items[menu_index]);
        }
    }
}

fn convert_to_string<const N: usize>(value: impl ufmt::uDisplay) -> heapless::String<N> {
    let mut string_buffer: heapless::String<N> = heapless::String::new();
    ufmt::uwrite!(&mut string_buffer, "{}", value).unwrap();
    string_buffer
}
