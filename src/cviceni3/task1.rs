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

    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    let x_axis = pins.a0.into_analog_input(&mut adc);
    let y_axis = pins.a1.into_analog_input(&mut adc);

    led_r.enable();
    led_g.enable();
    led_b.enable();

    let max_duty_r = led_r.get_max_duty();
    let max_duty_g = led_g.get_max_duty();
    let max_duty_b = led_b.get_max_duty();

    const X_CENTER: u16 = 495;
    const Y_CENTER: u16 = 506;

    loop {
        //led_r.toggle();
        let x_value = x_axis.analog_read(&mut adc);
        let y_value = y_axis.analog_read(&mut adc);

        ufmt::uwriteln!(&mut serial, "X: {}, Y: {}", x_value, y_value).unwrap();

        let red_intensity = if x_value > X_CENTER {
            ((x_value - X_CENTER) * 255 / (1024 - X_CENTER)) as u8
        } else {
            0u8
        };

        let green_intensity = if x_value < X_CENTER {
            ((X_CENTER - x_value) * 255 / X_CENTER) as u8
        } else {
            0u8
        };

        let blue_intensity = if y_value < Y_CENTER {
            ((Y_CENTER - y_value) * 255 / Y_CENTER) as u8
        } else {
            0u8
        } ;

        ufmt::uwriteln!(&mut serial, "RGB: ({}, {}, {})", red_intensity, green_intensity, blue_intensity).unwrap();

        led_r.set_duty(red_intensity);
        led_g.set_duty(green_intensity);
        led_b.set_duty(blue_intensity);

        arduino_hal::delay_ms(10);
    }
}
