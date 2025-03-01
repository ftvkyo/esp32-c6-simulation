#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock, delay::Delay, i2c::master::{Config, I2c}, time::RateExtU32,
};

use ssd1306::{prelude::DisplayRotation, size::DisplaySize128x64, I2CDisplayInterface, Ssd1306, mode::DisplayConfig};

use embedded_graphics::{image::{Image, ImageRaw}, pixelcolor::BinaryColor, prelude::*};

use log::{info, LevelFilter};


pub const W: usize = 128;
pub const H: usize = 64;
pub const FPS: u32 = 60;
pub const POINTS: usize = 10;
pub const CX: f32 = (W / 2) as f32;
pub const CY: f32 = (H / 2) as f32;


pub struct Img([u8; W * H / 8]);

impl Img {
    pub fn new() -> Self {
        Self([0; W * H / 8])
    }

    pub fn zero(&mut self) {
        for byte in &mut self.0 {
            *byte = 0;
        }
    }

    pub fn set(&mut self, x: isize, y: isize, value: bool) {
        if x < 0 || x >= W as isize {
            return;
        }

        if y < 0 || y >= H as isize {
            return;
        }

        let x = x as usize;
        let y = y as usize;

        let i_byte = (x + y * W) / 8;
        let i_bit = (x + y * W) % 8;

        let mask = 0b1 << (7 - i_bit);

        if value {
            self.0[i_byte] |= mask;
        } else {
            self.0[i_byte] &= !mask;
        }
    }

    pub fn data(&self) -> &[u8; W * H / 8] {
        &self.0
    }
}


#[derive(Clone)]
pub struct Point {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: [x, y],
            velocity: [0.0, 0.0],
        }
    }

    pub fn pos(&self) -> [isize; 2] {
        [
            self.position[0] as isize,
            self.position[1] as isize,
        ]
    }
}


#[derive(Clone)]
pub struct Points([Point; POINTS]);

impl Points {
    pub fn new() -> Self {
        let points = [
            Point::new(CX, CY - 20.0),
            Point::new(CX, CY - 10.0),
            Point::new(CX, CY),
            Point::new(CX, CY + 10.0),
            Point::new(CX, CY + 20.0),
            Point::new(CX + 30.0, CY + 5.0),
            Point::new(CX - 30.0, CY - 5.0),
            Point::new(CX + 20.0, CY),
            Point::new(CX - 20.0, CY),
            Point::new(CX + 50.0, CY - 15.0),
        ];

        Self(points)
    }

    pub fn step(&mut self, delta_ms: u32) {
        const G: f32 = 0.03;
        const F: f32 = 100.0;

        let mut points_after = self.0.clone();

        let mut center_of_mass = [0.0, 0.0];

        for p in &points_after {
            center_of_mass[0] += p.position[0] - CX;
            center_of_mass[1] += p.position[1] - CY;
        }

        for (pi, p) in points_after.iter_mut().enumerate() {
            p.position[0] -= center_of_mass[0] * delta_ms as f32 / 1000.0;
            p.position[1] -= center_of_mass[1] * delta_ms as f32 / 1000.0;

            for (oi, o) in self.0.iter().enumerate() {
                if pi == oi {
                    continue;
                }

                let dx = o.position[0] - p.position[0];
                let dy = o.position[1] - p.position[1];
                let m2 = (dx * dx + dy * dy) * F;

                p.velocity[0] += G * dx / m2 * delta_ms as f32;
                p.velocity[1] += G * dy / m2 * delta_ms as f32;
            }

            p.position[0] += p.velocity[0] * delta_ms as f32;
            p.position[1] += p.velocity[1] * delta_ms as f32;
        }

        for p in &mut points_after {
            if (p.position[0] as isize) <= 0 || (p.position[0] as usize) >= W - 1 {
                p.velocity[0] = -p.velocity[0];

                p.position[0] += p.velocity[0];
            }

            if (p.position[1] as isize) <= 0 || (p.position[1] as usize) >= H - 1 {
                p.velocity[1] = -p.velocity[1];

                p.position[1] += p.velocity[1];
            }
        }

        self.0 = points_after;
    }

    pub fn render(&self, img: &mut Img) {
        img.zero();

        for p in &self.0 {
            let [px, py] = p.pos();


            img.set(px - 1, py, true);
            img.set(px, py - 1, true);
            img.set(px, py, true);
            img.set(px + 1, py, true);
            img.set(px, py + 1, true);
        }
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

    let mut points = Points::new();
    let mut img = Img::new();

    loop {
        points.render(&mut img);

        let img_raw: ImageRaw<BinaryColor> = ImageRaw::new(img.data(), W as u32);
        let img = Image::new(&img_raw, Default::default());
        img.draw(&mut display).unwrap();
        display.flush().unwrap();

        delay.delay_millis(1_000 / FPS);
        points.step(1_000 / FPS);
    }
}
