use crate::{Ili9341, Interface, OutputPin};

use core::fmt::Debug;

use embedded_graphics::{
    drawable::Pixel,
    pixelcolor::Rgb565,
    Drawing,
};

impl<IfaceE, PinE, IFACE, RESET> Drawing<Rgb565> for Ili9341<IFACE, RESET>
where
    IFACE: Interface<Error = IfaceE>,
    RESET: OutputPin<Error = PinE>,
    IfaceE: Debug,
    PinE: Debug,
{
    fn draw<T>(&mut self, item_pixels: T)
    where
        T: IntoIterator<Item = Pixel<Rgb565>>,
    {
        for Pixel(pos, color) in item_pixels {
            self.draw_raw(
                pos.0 as u16,
                pos.1 as u16,
                pos.0 as u16,
                pos.1 as u16,
                &[color.0],
            )
            .expect("Failed to communicate with device");
        }
    }
}

