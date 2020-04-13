#![no_std]

#[cfg(feature = "graphics")]
extern crate embedded_graphics;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

pub mod spi;
use spi::SpiInterface;

pub mod gpio;

/// Trait representing the interface to the hardware.
///
/// Intended to abstract the various buses (SPI, MPU 8/9/16-bit) from the Controller code.
pub trait Interface {
    type Error;

    /// Sends a command with a sequence of 8-bit arguments
    ///
    /// Mostly used for sending configuration commands and flushing the framebuffer
    fn write(&mut self, command: u8, data: &[u8]) -> Result<(), Self::Error>;

}

const WIDTH: usize = 240;
const HEIGHT: usize = 320;

pub const BUFFER_SIZE: usize = WIDTH * HEIGHT * 2;

#[derive(Debug)]
pub enum Error<IfaceE, PinE> {
    Interface(IfaceE),
    OutputPin(PinE),
}

impl<IfaceE, PinE> From<IfaceE> for Error<IfaceE, PinE> {
    fn from(e: IfaceE) -> Self {
        Error::Interface(e)
    }
}

/// The default orientation is Portrait
pub enum Orientation {
    Portrait,
    PortraitFlipped,
    Landscape,
    LandscapeFlipped,
    LandscapeMirrored, // used for M5Stack
}

/// There are two method for drawing to the screen:
/// [draw_raw](struct.Ili9341.html#method.draw_raw) and
/// [draw_iter](struct.Ili9341.html#method.draw_iter).
///
/// In both cases the expected pixel format is rgb565.
///
/// The hardware makes it efficient to draw rectangles on the screen.
///
/// What happens is the following:
///
/// - A drawing window is prepared (with the 2 opposite corner coordinates)
/// - The starting point for drawint is the top left corner of this window
/// - Every pair of bytes received is intepreted as a pixel value in rgb565
/// - As soon as a pixel is received, an internal counter is incremented,
///   and the next word will fill the next pixel (the adjacent on the right, or
///   the first of the next row if the row ended)
pub struct Ili9341<'a, IFACE, RESET> {
    interface: IFACE,
    reset: RESET,
    width: usize,
    height: usize,
    buffer: &'a mut [u8; BUFFER_SIZE],
}

impl<'a, SpiE, PinE, SPI, CS, DC, RESET> Ili9341<'a, SpiInterface<SPI, CS, DC>, RESET>
where
    SPI: Transfer<u8, Error = SpiE> + Write<u8, Error = SpiE>,
    CS: OutputPin<Error = PinE>,
    DC: OutputPin<Error = PinE>,
    RESET: OutputPin<Error = PinE>,
{
    pub fn new_spi<DELAY: DelayMs<u16>>(
        spi: SPI,
        cs: CS,
        dc: DC,
        reset: RESET,
        delay: &mut DELAY,
        max_transfer_size: usize,
        buffer: &'a mut [u8; BUFFER_SIZE],
    ) -> Result<Self, Error<SpiE, PinE>> {
        let interface = SpiInterface::new(spi, cs, dc, max_transfer_size);
        Self::new(interface, reset, delay, buffer).map_err(|e| match e {
            Error::Interface(inner) => inner,
            Error::OutputPin(inner) => Error::OutputPin(inner),
        })
    }
}

impl<'a, IfaceE, PinE, IFACE, RESET> Ili9341<'a, IFACE, RESET>
where
    IFACE: Interface<Error = IfaceE>,
    RESET: OutputPin<Error = PinE>,
{
    pub fn new<DELAY: DelayMs<u16>>(
        interface: IFACE,
        reset: RESET,
        delay: &mut DELAY,
        buffer: &'a mut [u8; BUFFER_SIZE],
    ) -> Result<Self, Error<IfaceE, PinE>> {

        assert_eq!(BUFFER_SIZE, buffer.len());

        let mut ili9341 = Ili9341 {
            interface,
            reset,
            width: WIDTH,
            height: HEIGHT,
            buffer
        };

        ili9341.hard_reset(delay).map_err(Error::OutputPin)?;
        ili9341.command(Command::SoftwareReset, &[])?;
        delay.delay_ms(200);

        ili9341.command(Command::PowerControlA, &[0x39, 0x2c, 0x00, 0x34, 0x02])?;
        ili9341.command(Command::PowerControlB, &[0x00, 0xc1, 0x30])?;
        ili9341.command(Command::DriverTimingControlA, &[0x85, 0x00, 0x78])?;
        ili9341.command(Command::DriverTimingControlB, &[0x00, 0x00])?;
        ili9341.command(Command::PowerOnSequenceControl, &[0x64, 0x03, 0x12, 0x81])?;
        ili9341.command(Command::PumpRatioControl, &[0x20])?;
        ili9341.command(Command::PowerControl1, &[0x23])?;
        ili9341.command(Command::PowerControl2, &[0x10])?;
        ili9341.command(Command::VCOMControl1, &[0x3e, 0x28])?;
        ili9341.command(Command::VCOMControl2, &[0x86])?;
        ili9341.command(Command::MemoryAccessControl, &[0x48])?;
        ili9341.command(Command::PixelFormatSet, &[0x55])?;
        ili9341.command(Command::FrameControlNormal, &[0x00, 0x18])?;
        ili9341.command(Command::DisplayFunctionControl, &[0x08, 0x82, 0x27])?;
        ili9341.command(Command::Enable3G, &[0x00])?;
        ili9341.command(Command::GammaSet, &[0x01])?;
        ili9341.command(
            Command::PositiveGammaCorrection,
            &[
                0x0f, 0x31, 0x2b, 0x0c, 0x0e, 0x08, 0x4e, 0xf1, 0x37, 0x07, 0x10, 0x03, 0x0e, 0x09,
                0x00,
            ],
        )?;
        ili9341.command(
            Command::NegativeGammaCorrection,
            &[
                0x00, 0x0e, 0x14, 0x03, 0x11, 0x07, 0x31, 0xc1, 0x48, 0x08, 0x0f, 0x0c, 0x31, 0x36,
                0x0f,
            ],
        )?;
        ili9341.command(Command::SleepOut, &[])?;
        delay.delay_ms(120);
        ili9341.command(Command::DisplayOn, &[])?;

        Ok(ili9341)
    }

    fn hard_reset<DELAY: DelayMs<u16>>(&mut self, delay: &mut DELAY) -> Result<(), PinE> {
        // set high if previously low
        self.reset.set_high()?;
        delay.delay_ms(120);
        // set low for reset
        self.reset.set_low()?;
        delay.delay_ms(120);
        // set high for normal operation
        self.reset.set_high()?;
        delay.delay_ms(120);
        Ok(())
    }

    fn command(&mut self, cmd: Command, args: &[u8]) -> Result<(), IFACE::Error> {
        self.interface.write(cmd as u8, args)
    }

    fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), IFACE::Error> {
        self.command(
            Command::ColumnAddressSet,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xff) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xff) as u8,
            ],
        )?;
        self.command(
            Command::PageAddressSet,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xff) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xff) as u8,
            ],
        )?;
        Ok(())
    }

    /// Change the orientation of the screen
    pub fn set_orientation(&mut self, mode: Orientation) -> Result<(), IFACE::Error> {
        match mode {
            Orientation::Portrait => {
                self.width = WIDTH;
                self.height = HEIGHT;
                self.command(Command::MemoryAccessControl, &[0x40 | 0x08])
            }
            Orientation::Landscape => {
                self.width = HEIGHT;
                self.height = WIDTH;
                self.command(Command::MemoryAccessControl, &[0x20 | 0x08])
            }
            Orientation::PortraitFlipped => {
                self.width = WIDTH;
                self.height = HEIGHT;
                self.command(Command::MemoryAccessControl, &[0x80 | 0x08])
            }
            Orientation::LandscapeFlipped => {
                self.width = HEIGHT;
                self.height = WIDTH;
                self.command(Command::MemoryAccessControl, &[0x40 | 0x80 | 0x20 | 0x08])
            }
            Orientation::LandscapeMirrored => {
                self.width = HEIGHT;
                self.height = WIDTH;
                self.command(Command::MemoryAccessControl, &[0x08])
            }
        }
    }

    /// Set to invert colors (use inverted = true with M5Stack)
    pub fn set_inverted(&mut self, inverted: bool) -> Result<(), IFACE::Error> {
        match inverted {
            false => self.command(Command::InvertOff, &[]),
            true => self.command(Command::InvertOn, &[])
        }
    }

    /// Get the current screen width. It can change based on the current orientation
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the current screen height. It can change based on the current orientation
    pub fn height(&self) -> usize {
        self.height
    }

    /// Set the pixel at coordinates (x,y) to the given color (Rgb565)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: u16) {
        if x >= self.width || y >= self.height {
            return;
        }
        let pos_in_buffer: usize = ((y * self.width + x) * 2) as usize;
        let color_bytes = color.to_be_bytes();
        self.buffer[pos_in_buffer] = color_bytes[0];
        self.buffer[pos_in_buffer + 1] = color_bytes[1];
    }

    /// Set the whole screen to the given color (Rgb565)
    pub fn fill_screen(&mut self, color: u16) {

        let color_bytes = color.to_be_bytes();

        assert_eq!(BUFFER_SIZE, self.buffer.len());

        for i in 0..BUFFER_SIZE-1 {
            self.buffer[i] = color_bytes[i%2];
        }
    }
    
    /// Transfers the current frame from the buffer to the display.
    pub fn flush(&mut self) -> Result<(), IFACE::Error> {
        self.set_window(0, 0, self.width as u16, self.height as u16)?;
        self.interface.write(Command::MemoryWrite as u8, self.buffer)
    }

}

#[cfg(feature = "graphics")]
mod graphics;

#[derive(Clone, Copy)]
enum Command {
    SoftwareReset = 0x01,
    PowerControlA = 0xcb,
    PowerControlB = 0xcf,
    DriverTimingControlA = 0xe8,
    DriverTimingControlB = 0xea,
    PowerOnSequenceControl = 0xed,
    PumpRatioControl = 0xf7,
    PowerControl1 = 0xc0,
    PowerControl2 = 0xc1,
    VCOMControl1 = 0xc5,
    VCOMControl2 = 0xc7,
    MemoryAccessControl = 0x36,
    PixelFormatSet = 0x3a,
    FrameControlNormal = 0xb1,
    DisplayFunctionControl = 0xb6,
    Enable3G = 0xf2,
    GammaSet = 0x26,
    PositiveGammaCorrection = 0xe0,
    NegativeGammaCorrection = 0xe1,
    SleepOut = 0x11,
    DisplayOn = 0x29,
    ColumnAddressSet = 0x2a,
    PageAddressSet = 0x2b,
    MemoryWrite = 0x2c,
    InvertOff = 0x20,
    InvertOn = 0x21,

}
