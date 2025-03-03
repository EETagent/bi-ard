#![no_std]
#![no_main]

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm, Timer2Pwm};
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);
    let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);


    let mut led_g = pins.d11.into_output().into_pwm(&timer2);
    let mut led_b = pins.d10.into_output().into_pwm(&timer1);
    let mut led_r = pins.d9.into_output().into_pwm(&timer1);


    let mut buzzer = pins.d7.into_output();

    let button = pins.d2.into_pull_up_input();

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let x_axis = pins.a0.into_analog_input(&mut adc);
    let y_axis = pins.a1.into_analog_input(&mut adc);


    let potenciometer = pins.a2.into_analog_input(&mut adc);

    led_r.enable();
    led_g.enable();
    led_b.enable();


    enum Color {
        Red,
        Green,
        Blue,
        None
    }

    let mut color = Color::None;

    loop {

        let x_value = x_axis.analog_read(&mut adc);
        let y_value = y_axis.analog_read(&mut adc);

        let potenciometer_value = potenciometer.analog_read(&mut adc);

        //let potenciometer_value = potenciometer_value.saturating_sub(490).saturating_mul(5) as u8;

        if x_value < 100 {
            color = Color::Red;
        } else if x_value > 900 {
            color = Color::Blue;
        } else if y_value < 100 {
            color = Color::Green;
        } else {
            if button.is_low() {
                buzzer.set_high();
                arduino_hal::delay_ms(2);
                buzzer.set_low();
            }
            color = Color::None;
        }

        ufmt::uwriteln!(&mut serial, "X: {}, Y: {} P: {}", x_value, y_value, potenciometer_value).unwrap();


        led_r.set_duty(if matches!(color, Color::Red) {
            potenciometer_value as u8
        } else {
            0
        });

        led_g.set_duty(if matches!(color, Color::Green) {
            potenciometer_value as u8
        } else {
            0
        });

        led_b.set_duty(if matches!(color, Color::Blue) {
            potenciometer_value as u8
        } else {
            0
        });

        arduino_hal::delay_ms(10);
    }
}
