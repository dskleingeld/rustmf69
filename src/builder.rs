use core::marker::PhantomData;
use core::fmt::Debug;
use core::num::NonZeroU8;

use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::blocking::delay::{DelayMs, DelayUs};

use crate::{Radio, AddressFiltering, RegisterFlags, FreqencyBand, Bitrate, RadioMode, PackageLength};

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
pub struct RadioBuilder<SPI, CS, DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,

	FREQSET: ToAssign,
	NODEIDSET: ToAssign,
	PACKAGELENSET: ToAssign,
{
	spi: SPI,
	cs: CS,
	delay: DELAY,

	freq_set: PhantomData<FREQSET>,
	node_id_set: PhantomData<NODEIDSET>,
	package_len_set: PhantomData<PACKAGELENSET>,

	freq_band: FreqencyBand, //non optional

	network_id: Option<NonZeroU8>,
	node_address: u8,
	broadcast_addr: Option<u8>,

	bitrate: Option<Bitrate>,   //optional (default = smthg)
	package_len: Option<PackageLength>,
	power_level: Option<u8>, //optional (default, max)
}

impl<SPI, CS, DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET> RadioBuilder<SPI, CS, DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,
	E : core::fmt::Debug,

	FREQSET: ToAssign,
	NODEIDSET: ToAssign,
	PACKAGELENSET: ToAssign,
{
	pub fn adress(self, adress: u8) -> RadioBuilder<SPI,CS,DELAY, E, FREQSET, Yes, PACKAGELENSET> {
		RadioBuilder {
			node_address: adress,

			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_set: PhantomData,
			node_id_set: PhantomData,
			package_len_set: PhantomData,

			freq_band: self.freq_band,
			broadcast_addr: None,
			network_id: self.network_id,
			bitrate: self.bitrate,
			package_len: self.package_len,
			power_level: self.power_level,
		}
	}

	pub fn freqency_band(self, freq_band: FreqencyBand) -> RadioBuilder<SPI,CS,DELAY, E, Yes, NODEIDSET, PACKAGELENSET> {
		RadioBuilder {
			freq_band: freq_band,

			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_set: PhantomData,
			node_id_set: PhantomData,
			package_len_set: PhantomData,

			node_address: self.node_address,
			broadcast_addr: None,
			network_id: self.network_id,
			bitrate: self.bitrate,
			package_len: self.package_len,
			power_level: self.power_level,
		}
	}

	pub fn fixed_package_length(self, len: u8) -> RadioBuilder<SPI,CS,DELAY, E, Yes, NODEIDSET, Yes> {
		RadioBuilder {

			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_set: PhantomData,
			node_id_set: PhantomData,
			package_len_set: PhantomData,

			freq_band: self.freq_band,
			node_address: self.node_address,
			broadcast_addr: None,
			network_id: self.network_id,
			bitrate: self.bitrate,
			package_len: Some(PackageLength::Fixed(len)),
			power_level: self.power_level,
		}
	}

	pub fn max_package_length(self, len: u8) -> RadioBuilder<SPI,CS,DELAY, E, Yes, NODEIDSET, Yes> {
		RadioBuilder {


			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq_set: PhantomData,
			node_id_set: PhantomData,
			package_len_set: PhantomData,

			freq_band: self.freq_band,
			node_address: self.node_address,
			broadcast_addr: None,
			network_id: self.network_id,
			bitrate: self.bitrate,
			package_len: Some(PackageLength::Max(len)),
			power_level: self.power_level,
		}
	}

	pub fn broadcast(self, broadcast_adress: u8) -> RadioBuilder<SPI,CS,DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET> {

		RadioBuilder {
			broadcast_addr: Some(broadcast_adress),
			..self
		}
	}
	pub fn network_id(self, network_id: NonZeroU8) -> RadioBuilder<SPI,CS,DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET> {
		RadioBuilder {
			network_id: Some(network_id),
			..self
		}
	}
	pub fn bitrate(self, bitrate: Bitrate) -> RadioBuilder<SPI,CS,DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET> {
		RadioBuilder {
			bitrate: Some(bitrate),
			..self
		}
	}
	pub fn power_level(self, power_level: u8) -> RadioBuilder<SPI,CS,DELAY, E, FREQSET, NODEIDSET, PACKAGELENSET> {
		RadioBuilder {
			power_level: Some(power_level),
			..self
		}
	}
}

pub fn radio<SPI,CS,DELAY,E>(spi: SPI, cs: CS, delay: DELAY) -> RadioBuilder<SPI,CS,DELAY, E, No, No, No>
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
		package_len_set: PhantomData,

		//will be set anyway, thus doesnt really matter
		freq_band: FreqencyBand::ISM433mhz,
		node_address: 0,
		broadcast_addr: None,
		network_id: None,
		bitrate: None,
		package_len: None,
		power_level: None,
  }
}

fn default_band_freq(freq_band: &FreqencyBand) -> u32 {
	match freq_band {
		FreqencyBand::ISM315mhz => 315_000_000,
		FreqencyBand::ISM433mhz => 433_000_000,
		FreqencyBand::ISM868mhz => 868_000_000,
		FreqencyBand::ISM915mhz => 915_000_000,
	}
}

impl<SPI, CS, DELAY, E, PACKAGELENSET> RadioBuilder<SPI, CS, DELAY, E, Yes, Yes, PACKAGELENSET>
where
	SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
	DELAY: DelayMs<u16>+DelayUs<u16>,
	CS: OutputPin,
	E : core::fmt::Debug,
	PACKAGELENSET: ToAssign,
 {

	pub fn build(self) -> Radio<SPI, CS, DELAY> {

		let adress_filtering = if let Some(broadcast_addr) = self.broadcast_addr {
			AddressFiltering::AddressOrBroadcast((self.node_address, broadcast_addr))
		} else {
			AddressFiltering::AddressOnly(self.node_address)
		};

		Radio {
			spi: self.spi,
			cs: self.cs,
			delay: self.delay,

			freq: default_band_freq(&self.freq_band),
			bitrate: self.bitrate.unwrap_or(Bitrate::default()),   //optional (default = smthg)
			power_level: self.power_level.unwrap_or(31), //optional (default, max)

			network_filtering: self.network_id,
			adress_filtering: adress_filtering,

			mode: RadioMode::Standby,
			package_len: self.package_len.unwrap_or(PackageLength::default()),

			register_flags: RegisterFlags::default(),
		}
	}
}
