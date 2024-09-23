#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};

use log::{info, LevelFilter};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    esp_println::logger::init_logger(LevelFilter::Info);

    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    info!("Initialised");

    let mut counter: u8 = 0;

    let mut led = Output::new(io.pins.gpio7, Level::Low);
    led.set_high();

    loop {
        led.toggle();
        info!("LED toggled; Counter: {}", counter);
        counter += 1;
        delay.delay_millis(500u32);
    }
}
