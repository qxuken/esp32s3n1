use std::sync::{Arc, Mutex};

use anyhow::Result;
use colors_transform::{Color, Rgb};
use rgb::RGB8;

use crate::rgb_led::WS2812RMT;

#[derive(Debug)]
pub struct AppState<'a> {
    rgb: WS2812RMT<'a>,
    pub rgb_val: Rgb,
}

impl<'a> AppState<'a> {
    pub fn try_new(mut rgb: WS2812RMT<'a>, rgb_val: Rgb) -> Result<Self> {
        rgb.set_pixel(RGB8::new(
            rgb_val.get_red() as u8,
            rgb_val.get_green() as u8,
            rgb_val.get_blue() as u8,
        ))?;
        Ok(Self { rgb, rgb_val })
    }
}

impl<'a> AppState<'a> {
    pub fn shared(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    pub fn set_rgb(&mut self, rgb: Rgb) -> Result<()> {
        log::info!("new color to set {:?}", rgb);
        let val = RGB8::new(
            rgb.get_red() as u8,
            rgb.get_green() as u8,
            rgb.get_blue() as u8,
        );
        self.rgb.set_pixel(val)?;
        self.rgb_val = rgb;
        Ok(())
    }
}
