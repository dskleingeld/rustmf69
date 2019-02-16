use core::marker::PhantomData;
//use core::option::Option;
use core::fmt::Debug;

use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::delay::{DelayMs, DelayUs};

use crate::{Radio, FreqencyBand, Bitrate, RadioMode};

#[derive(Debug, Default)]
pub struct Yes;
#[derive(Debug, Default)]
pub struct No;

pub trait ToAssign: Debug {}
pub trait Assigned: ToAssign {}
pub trait NotAssigned: ToAssign {}

impl ToAssign for Yes {}
impl ToAssign for No {}

impl Assigned for Yes {}
impl NotAssigned for No {}

#[derive(Debug, Clone)]
pub struct RadioBuilder<SPI, CS, DELAY, E, FREQ_SET, NODE_ID_SET>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,

	FREQ_SET: ToAssign,
	NODE_ID_SET: ToAssign,
{
	spi: SPI,
	cs: CS,
	delay: DELAY,

	freq_set: PhantomData<FREQ_SET>,
	node_id_set: PhantomData<NODE_ID_SET>,

	freq_band: FreqencyBand, //non optional
	node_id: u8,   //non optional
	network_id: Option<u8>,//optional (default = 0)
	bitrate: Option<Bitrate>,   //optional (default = smthg)
	power_level: Option<u8>, //optional (default, max)
}

impl<SPI, CS, DELAY, E, FREQ_SET, NODE_ID_SET> RadioBuilder<SPI, CS, DELAY, E, FREQ_SET, NODE_ID_SET>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,
	E : core::fmt::Debug,

	FREQ_SET: ToAssign,
	NODE_ID_SET: ToAssign,
{
	pub fn node_id(self, node_id: u8) -> RadioBuilder<SPI,CS,DELAY, E, FREQ_SET, Yes> {
		RadioBuilder {
			node_id: node_id,

			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_set: PhantomData,
			node_id_set: PhantomData,

			freq_band: self.freq_band,
			network_id: self.network_id,
			bitrate: self.bitrate,
			power_level: self.power_level,
		}
	}
	pub fn freqency_band(self, freq_band: FreqencyBand) -> RadioBuilder<SPI,CS,DELAY, E, Yes, NODE_ID_SET> {
		RadioBuilder {
			freq_band: freq_band,

			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_set: PhantomData,
			node_id_set: PhantomData,

			node_id: self.node_id,
			network_id: self.network_id,
			bitrate: self.bitrate,
			power_level: self.power_level,
		}
	}

	pub fn network_id(self, network_id: u8) -> RadioBuilder<SPI,CS,DELAY, E, FREQ_SET, NODE_ID_SET> {
		RadioBuilder {
			network_id: Some(network_id),
			..self
		}
	}
	pub fn bitrate(self, bitrate: Bitrate) -> RadioBuilder<SPI,CS,DELAY, E, FREQ_SET, NODE_ID_SET> {
		RadioBuilder {
			bitrate: Some(bitrate),
			..self
		}
	}
	pub fn power_level(self, power_level: u8) -> RadioBuilder<SPI,CS,DELAY, E, FREQ_SET, NODE_ID_SET> {
		RadioBuilder {
			power_level: Some(power_level),
			..self
		}
	}
}

pub fn radio<SPI,CS,DELAY,E>(spi: SPI, cs: CS, delay: DELAY) -> RadioBuilder<SPI,CS,DELAY, E, No, No>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,
	E : core::fmt::Debug,
 {
  RadioBuilder {
		spi: spi,
		cs: cs,
		delay: delay,

		freq_set: PhantomData,
		node_id_set: PhantomData,

		//will be set anyway, thus doesnt really matter
		freq_band: FreqencyBand::ISM433mhz,
		node_id: 0,
		network_id: None,
		bitrate: None,
		power_level: None,
  }
}

impl<SPI, CS, DELAY, E> RadioBuilder<SPI, CS, DELAY, E, Yes, Yes>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,
	E : core::fmt::Debug,
 {

	pub fn build(self) -> Radio<SPI, CS, DELAY> {
		Radio {
			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_band: self.freq_band, //non optional
			node_id: self.node_id,   //non optional
			network_id: self.network_id.unwrap_or(0),//optional (default = 0)
			bitrate: self.bitrate.unwrap_or(Bitrate::default()),   //optional (default = smthg)
			power_level: self.power_level.unwrap_or(31), //optional (default, max)

			mode: RadioMode::Standby,
		}
	}
}
