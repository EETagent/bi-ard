#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

static LED_IS_OFF: AtomicBool = AtomicBool::new(false);
static LED_COLOR: AtomicU8 = AtomicU8::new(0);
static PIN_CHANGED: AtomicBool = AtomicBool::new(false);

// Mapping: 0 => red (D13 -> PB5), 1 => green (D12 -> PB4), 2 => blue (D11 -> PB3)
fn color_to_state(color: u8) -> u8 {
    match color {
        0 => 1 << 5, // red: set PB5 high
        1 => 1 << 4, // green: set PB4 high
        2 => 1 << 3, // blue: set PB3 high
        _ => 0,
    }
}

fn debounce_delay() {
    for _ in 0..32000 {
        unsafe { core::ptr::read_volatile(&0u8) };
    }
}

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT2() {
    debounce_delay();
    let pd2_state = unsafe { (*avr_device::atmega328p::PORTD::ptr()).pind.read().bits() } & (1 << 2);
    if pd2_state != 0 {
        return;
    }

    PIN_CHANGED.store(true, Ordering::SeqCst);

    let current = LED_COLOR.load(Ordering::SeqCst);
    let next = (current + 1) % 3;
    LED_COLOR.store(next, Ordering::SeqCst);

    let new_state = color_to_state(next);
    if !LED_IS_OFF.load(Ordering::SeqCst) {
        unsafe {
            (*avr_device::atmega328p::PORTB::ptr()).portb.write(|w| w.bits(new_state));
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let _joystick_up = pins.d2.into_floating_input().downgrade();

    dp.EXINT.pcicr.write(|w| unsafe { w.bits(0b100) });
    dp.EXINT.pcmsk2.write(|w| w.bits(0b100));

    unsafe { avr_device::interrupt::enable() };

    loop {
        let color = LED_COLOR.load(Ordering::SeqCst);
        let new_state = color_to_state(color);
        unsafe {
            (*avr_device::atmega328p::PORTB::ptr()).portb.write(|w| w.bits(new_state));
        }
        LED_IS_OFF.store(false, Ordering::SeqCst);

        arduino_hal::delay_ms(1000);

        unsafe {
            (*avr_device::atmega328p::PORTB::ptr()).portb.write(|w| w.bits(0));
        }

        LED_IS_OFF.store(true, Ordering::SeqCst);

        arduino_hal::delay_ms(1000);
    }
}
