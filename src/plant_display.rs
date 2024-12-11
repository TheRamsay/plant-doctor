use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::MonoTextStyleBuilder,
    pixelcolor::{raw::BigEndian, BinaryColor},
    prelude::Point,
    text::{Text, TextStyleBuilder},
};
use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use epd_waveshare::{
    color::Color,
    epd2in9_v2::{Display2in9, Epd2in9},
    prelude::WaveshareDisplay,
};

use embedded_graphics::prelude::*;

pub struct DisplayInput {
    pub plant_name: String,
    pub soil_moisture: f32,
    pub light_intensity: f32,
    // Currently not used :/
    // pub air_temperature: f32,
    // pub air_humidity: f32,
}

pub struct PlantDisplay<SPI, BUSY, DC, RST, DELAY> {
    epd: Epd2in9<SPI, BUSY, DC, RST, DELAY>,
    display: Display2in9,
    delay: DELAY,
    device: SPI,
}

impl<SPI, BUSY, DC, RST, DELAY> PlantDisplay<SPI, BUSY, DC, RST, DELAY>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DELAY: DelayNs,
{
    pub fn new(
        epd: Epd2in9<SPI, BUSY, DC, RST, DELAY>,
        display: Display2in9,
        delay: DELAY,
        device: SPI,
    ) -> Self {
        Self {
            epd,
            display,
            delay,
            device,
        }
    }

    pub fn display_input(&mut self, input: &DisplayInput) {
        self.display.clear(Color::White).unwrap();

        self.display
            .set_rotation(epd_waveshare::prelude::DisplayRotation::Rotate90);

        self.draw_text(&input.plant_name, 10, 10);

        self.draw_text(&format!("{:.2}%", input.soil_moisture), 15, 70);

        self.draw_text(&format!("{:.2} lux", input.light_intensity), 160, 70);

        self.epd
            .update_frame(&mut self.device, self.display.buffer(), &mut self.delay)
            .unwrap();
        self.epd
            .display_frame(&mut self.device, &mut self.delay)
            .expect("display frame new graphics");

        self.delay.delay_ms(2000);
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32) {
        let style = MonoTextStyleBuilder::new()
            .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
            .text_color(Color::Black)
            .background_color(Color::White)
            .build();

        let text_style = TextStyleBuilder::new()
            .baseline(embedded_graphics::text::Baseline::Top)
            .build();

        let _ = Text::with_text_style(text, Point::new(x, y), style, text_style)
            .draw(&mut self.display);
    }

    pub fn clear(&mut self) {
        self.display.clear(Color::White).unwrap();
    }

    pub fn Black(&mut self) {
        self.display.clear(Color::Black).unwrap();
    }

    // TODO: Implement this
    pub fn display_image<T>(&mut self, binary_image: &[u8], width: u32, height: u32) {
        let x = Image::new(
            &ImageRaw::<BinaryColor, BigEndian>::new(binary_image, width),
            Point::new(0, 0),
        );

        // x.draw(&mut self.display).unwrap();
    }
}
