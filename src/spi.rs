use crate::{Error, Interface};
use embedded_hal::blocking::spi;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::{Mode, Phase, Polarity};

/// SPI mode
pub const MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

/// `Interface` implementation for SPI interfaces
pub struct SpiInterface<SPI, CS, DC> {
    spi: SPI,
    cs: CS,
    dc: DC,
    max_transfer_size: usize,
}

impl<SPI, CS, DC, SpiE, PinE> SpiInterface<SPI, CS, DC>
where
    SPI: spi::Transfer<u8, Error = SpiE> + spi::Write<u8, Error = SpiE>,
    CS: OutputPin<Error = PinE>,
    DC: OutputPin<Error = PinE>,
{
    pub fn new(spi: SPI, cs: CS, dc: DC, max_transfer_size: usize) -> Self {
        Self { spi, cs, dc, max_transfer_size }
    }
}

impl<SPI, CS, DC, SpiE, PinE> Interface for SpiInterface<SPI, CS, DC>
where
    SPI: spi::Transfer<u8, Error = SpiE> + spi::Write<u8, Error = SpiE>,
    CS: OutputPin<Error = PinE>,
    DC: OutputPin<Error = PinE>,
{
    type Error = Error<SpiE, PinE>;

    fn write(&mut self, command: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::OutputPin)?;

        self.dc.set_low().map_err(Error::OutputPin)?;
        self.spi.write(&[command]).map_err(Error::Interface)?;

        self.dc.set_high().map_err(Error::OutputPin)?;

        for chunk in data.chunks(self.max_transfer_size) {
            self.spi.write(&chunk).map_err(Error::Interface)?;
        }

        self.cs.set_high().map_err(Error::OutputPin)?;
        Ok(())
    }

}
