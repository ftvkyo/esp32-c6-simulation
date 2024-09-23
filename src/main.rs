#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, gpio::Io, i2c::I2C, peripherals::Peripherals, prelude::*, system::SystemControl
};

use ssd1306::{prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306, mode::DisplayConfig};

use embedded_graphics::{image::{Image, ImageRaw}, pixelcolor::BinaryColor, prelude::*};

use log::{info, LevelFilter};


#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    esp_println::logger::init_logger(LevelFilter::Info);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio1,
        io.pins.gpio2,
        400.kHz(),
        &clocks,
    );

    info!("Initialised: controller");

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    display.init().unwrap();

    info!("Initialised: display");

    const W: usize = 128;
    const H: usize = 64;

    let mut img_data = [0u8; W * H];

    for row in 0..H {
        for col in 0..W {
            let i_byte = (row + col * H) / 8;
            let i_bit = (row + col * H) % 8;

            if row == col {
                img_data[i_byte] |= 0b1 << (7 - i_bit);
            }
        }
    }

    let img_raw: ImageRaw<BinaryColor> = ImageRaw::new(&img_data, 64);
    let img = Image::new(&img_raw, Point::default());

    img.draw(&mut display).unwrap();
    display.flush().unwrap();

    info!("Image drawn");

    loop {}
}
