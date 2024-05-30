
//! # Use ws2812 leds via spi
//!
//! - For usage with `smart-leds`
//! - Implements the `SmartLedsWrite` trait
//!
//! Needs a type implementing the `spi::SpiBus` trait.
//!
//! The spi peripheral should run at 2MHz to 3.8 MHz

// Timings for ws2812 from https://cpldcpu.files.wordpress.com/2014/01/ws2812_timing_table.png
// Timings for sk6812 from https://cpldcpu.wordpress.com/2016/03/09/the-sk6812-another-intelligent-rgb-led/

use embedded_hal::spi::SpiBus;
use core::marker::PhantomData;
use core::slice::from_ref;

use smart_leds_trait::{SmartLedsWrite, RGB8};

pub mod devices {
    pub struct Ws2812;
}

pub struct Ws2812<SPI, DEVICE = devices::Ws2812> {
    spi: SPI,
    device: PhantomData<DEVICE>,
}

impl<SPI, E> Ws2812<SPI>
where
    SPI: SpiBus<u8, Error = E>,
{
    /// Use ws2812 devices via spi
    ///
    /// The SPI bus should run within 2 MHz to 3.8 MHz
    ///
    /// You may need to look at the datasheet and your own hal to verify this.
    ///
    /// Please ensure that the mcu is pretty fast, otherwise weird timing
    /// issues will occur
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            device: PhantomData {},
        }
    }
}


impl<SPI, D, E> Ws2812<SPI, D>
where
    SPI: SpiBus<u8, Error = E>,
{

    fn write_byte(&mut self, mut data: u8) -> Result<(), E> {
        //0, 1
        let patterns = [0b1100_0000, 0b1111_0000];
        for _ in 0..8 {
            let bits = (data & 0b1000_0000) >> 7;
            self.spi.write(from_ref(&patterns[bits as usize]))?;
            data <<= 1;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), E> {
        // Should be > 300μs, so for an SPI Freq. of 3.8MHz, we have to send at least 1140 low bits or 140 low bytes
        for _ in 0..64 {
            self.spi.write(from_ref(&0))?;
        }
        Ok(())
    }
}

impl<SPI, E> SmartLedsWrite for Ws2812<SPI>
where
    SPI: SpiBus<u8, Error = E>,
{
    type Error = E;
    type Color = RGB8;
    /// Write all the items of an iterator to a ws2812 strip
    fn write<T, I>(&mut self, iterator: T) -> Result<(), E>
    where
        T: IntoIterator<Item = I>,
        I: Into<Self::Color>,
    {
        if cfg!(feature = "mosi_idle_high") {
            self.flush()?;
        }

        for item in iterator {
            let item = item.into();
            self.write_byte(item.r)?;
            self.write_byte(item.g)?;
            self.write_byte(item.b)?;
        }
        self.flush()?;
        Ok(())
    }
}

