use crate::{Ili9341, Interface, OutputPin};

use core::fmt::Debug;

use embedded_graphics::{
    coord::Coord,
    drawable::Pixel,
    drawable::Dimensions,
    pixelcolor::Rgb565,
    Drawing,
    SizedDrawing,
};

impl<'a, IfaceE, PinE, IFACE, RESET> Drawing<Rgb565> for Ili9341<'a, IFACE, RESET>
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
        for pixel in item_pixels {
            self.set_pixel(
                (pixel.0).0 as usize, // pixel.position.x
                (pixel.0).1 as usize, // pixel.position.y
                (pixel.1).0 // pixel.color.value
            );
        }
    }
}

impl<'a, IfaceE, PinE, IFACE, RESET> SizedDrawing<Rgb565> for Ili9341<'a, IFACE, RESET>
where
    IFACE: Interface<Error = IfaceE>,
    RESET: OutputPin<Error = PinE>,
    IfaceE: Debug,
    PinE: Debug,
{
    fn draw_sized<T>(&mut self, item: T)
    where
        T: IntoIterator<Item = Pixel<Rgb565>> + Dimensions,
    {
        let Coord { 0: x0, 1: y0 } = item.top_left();
        if x0 as usize >= self.width() || y0 as usize >= self.height() {
            return;
        }

        for pixel in item.into_iter() {
            self.set_pixel(
                (pixel.0).0 as usize, // pixel.position.x
                (pixel.0).1 as usize, // pixel.position.y
                (pixel.1).0 // pixel.color.value
            );
        }
        
    }
}