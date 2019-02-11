//working off example: https://github.com/japaric/mfrc522/blob/master/src/lib.rs
#![no_std]

extern crate embedded_hal as hal;
use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::delay::DelayMs;

pub struct Radio<SPI, D> {
	spi: SPI,
	delay: D,
	mode: RadioMode,
	power_level: u8, //TODO restrict max-min
}

enum RadioMode {
	Sleep = 0, // Xtal Off
	Standby = 1, // Xtal On
	Synth = 2, // Pll On
	Rx = 3, // Rx Mode
	Tx = 4, // Tx Mode
}

#[allow(dead_code)]
pub enum Bitrate { 
	Low, 
	High,
	Standard, 
}

#[allow(dead_code)]
pub enum FreqencyBand {
	ISM315mhz = 31, // Non Trivial Values To Avoid Misconfiguration
	ISM433mhz = 43,
	ISM868mhz = 86,
	ISM915mhz = 91,
}

impl<E, D, SPI> Radio<SPI, D> 
where SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
      D: DelayMs<u16> {

  pub fn new(spi: SPI, delay: D) ->	Result<Self, ()> {
    Ok(Radio { spi, delay, mode: RadioMode::Rx, power_level: 31 })
  }

	pub fn init(&mut self, freq_band: FreqencyBand, node_id: u8, network_id: u8, speed: Bitrate) {
		
	
	}


	// pub fn send(uint8_t toAddress, const void* buffer, uint8_t bufferSize, bool requestACK=false) {
	
	
	// }

}
