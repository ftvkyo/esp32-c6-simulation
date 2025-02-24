#![no_std]
#![no_main]

use core::f32::consts::PI;

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock, delay::Delay, i2c::master::{Config, I2c}, time::RateExtU32,
};

use ssd1306::{prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306, mode::DisplayConfig};

use embedded_graphics::{image::{Image, ImageRaw}, pixelcolor::BinaryColor, prelude::*};


use micromath::F32Ext;

use log::{info, LevelFilter};


pub const W: usize = 128;
pub const H: usize = 64;
pub const FPS: u32 = 60;

pub struct Img([u8; W * H / 8]);

impl Img {
    const POINTS: usize = 3;
    const SPEED: f32 = 0.01;

    // Phase shift between points
    const ANGLE: f32 = PI * 2.0 / Self::POINTS as f32;

    pub fn new() -> Self {
        Self([0; W * H / 8])
    }

    pub fn zero(&mut self) {
        for byte in &mut self.0 {
            *byte = 0;
        }
    }

    pub fn render_frame(&mut self, ms: u32) {
        let w = W as f32;
        let wc = w / 2.0;
        let h = H as f32;
        let hc = h / 2.0;

        let fx = 1.0;
        let fy = 2.0;

        self.zero();

        for point in 0..Self::POINTS {
            let shift_base = Self::ANGLE * point as f32;
            let shift = shift_base + ms as f32 * Self::SPEED;

            let px = (wc + (shift * fx).cos() * w / 3.0) as usize;
            let py = (hc + (shift * fy).sin() * h / 3.0) as usize;

            let i_byte = (px + py * W) / 8;
            let i_bit = (px + py * W) % 8;

            self.0[i_byte] |= 0b1 << (7 - i_bit);
        }
    }

    pub fn get(&self) -> &[u8; W * H / 8] {
        &self.0
    }
}


#[esp_hal::main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let delay = Delay::new();

    esp_println::logger::init_logger(LevelFilter::Info);

    let i2c = I2c::new(
        peripherals.I2C0,
        Config::default().with_frequency(400.kHz()),
    ).unwrap()
        .with_sda(peripherals.GPIO1)
        .with_scl(peripherals.GPIO2);

    info!("Initialised: controller");

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_buffered_graphics_mode();
    display.init().unwrap();

    info!("Initialised: display");

    let mut img_data = Img::new();
    let mut ms = 0;

    loop {
        img_data.render_frame(ms);

        let img_raw: ImageRaw<BinaryColor> = ImageRaw::new(img_data.get(), W as u32);
        let img = Image::new(&img_raw, Point::default());
        img.draw(&mut display).unwrap();
        display.flush().unwrap();

        delay.delay_millis(1_000 / FPS);
        ms += 1_000 / FPS;
    }
}
