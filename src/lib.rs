//working off example (spi): https://github.com/japaric/mfrc522/blob/master/src/lib.rs
//another example: https://github.com/JohnDoneth/hd44780-driver/blob/master/examples/raspberrypi/src/main.rs
//#![no_std] //FIXME TODO remove all std lib dependencies
extern crate embedded_hal as hal;

#[macro_use]
extern crate bitflags;

use core::num::NonZeroU8;
use core::cmp::min;

use hal::blocking::spi;
use hal::digital::OutputPin;
use hal::spi::{Mode, Phase, Polarity};
use hal::blocking::delay::{DelayMs, DelayUs};

mod registers;
use registers::{Register, RF69_FSTEP, FXOSC};
mod builder;
pub use builder::{RadioBuilder,radio};

pub struct Radio<SPI, CS, DELAY> {
	spi: SPI,
	cs: CS,
	delay: DELAY,

	freq: u32,
	bitrate: Bitrate,   //optional (default = smthg)
	power_level: u8, //optional (default, max)

	network_filtering: Option<NonZeroU8>,
	adress_filtering: AddressFiltering,
	encryption_key: Option<[u8;17]>,

	mode: RadioMode,
	package_len: PackageLength,

	register_flags: RegisterFlags
}

//local copy of register flags to save register read operations
struct RegisterFlags {
	mode: registers::OpMode,
	sync: registers::SyncConfig,
	config1: registers::PacketConfig1,
	config2: registers::PacketConfig2,
	pa_level: registers::PaLevel,
}

impl Default for RegisterFlags {
	fn default() -> Self {
		Self {
			mode: registers::OpMode::Standby
		          & !registers::OpMode::Sequencer_Off
		          & !registers::OpMode::Listen_On,
			sync: registers::SyncConfig::On
			      | registers::SyncConfig::Fifofill_Auto
			      | registers::SyncConfig::Size_2
			      | registers::SyncConfig::Tol_0,
			config1: registers::PacketConfig1::Format_Variable
			      | registers::PacketConfig1::Dcfree_Off
			      | registers::PacketConfig1::Crc_On
			      | registers::PacketConfig1::Crcautoclear_On
			      | registers::PacketConfig1::Adrsfiltering_Off,
			config2: registers::PacketConfig2::Rxrestartdelay_2bits
			      & !registers::PacketConfig2::Aes_On
			      | registers::PacketConfig2::Autorxrestart_On,
			pa_level: registers::PaLevel::Pa0_On
			      & !registers::PaLevel::Pa1_On
			      & !registers::PaLevel::Pa2_On,
		}
	}
}

#[allow(dead_code)]
enum AddressFiltering {
	None,
	AddressOnly(u8),
	AddressOrBroadcast((u8,u8)), //(addr, broadcast_addr)
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
enum RadioMode { //rename transeiver?
	Sleep = 0, // Xtal Off
	Standby = 1, // Xtal On
	FreqSynth = 2, // Pll On
	Rx = 3, // Rx Mode
	Tx = 4, // Tx Mode
}

impl Default for RadioMode {
	fn default() -> Self {
		RadioMode::Standby
	}
}

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub enum Bitrate { 
	Lowest,
	Low, 
	Standard,
	High,
	Custom(u32),
}

impl Default for Bitrate {
	fn default() -> Self {
		Bitrate::Standard
	}
}

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub enum PackageLength {
	Fixed(u8), //in bytes
	Max(u8),
}

impl Default for PackageLength {
	fn default() -> Self {
		PackageLength::Fixed(16)
	}
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FreqencyBand {
	ISM315mhz,
	ISM433mhz,
	ISM868mhz,
	ISM915mhz,
}

/// SPI mode
pub const SPI_MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

pub const SPI_SPEED: u32 = 500_000;

impl<SPI,CS, D, E> Radio<SPI, CS, D>
where SPI: spi::Transfer<u8, Error = E> + spi::Write<u8, Error = E>,
      D: DelayMs<u16>+DelayUs<u16>,
      CS: OutputPin,
	  E: core::fmt::Debug {

	fn configure_radio(&mut self) -> Result<(),&'static str> {
		self.set_default_config();
		self.set_package_filtering();
		self.set_bitrate();
		self.set_frequency()?;
		self.set_payload_length();
		self.set_power_level();
		self.set_encryption_key();
		Ok(())
	}


	pub fn init(&mut self) -> Result<(),&'static str> {
		//self.cs.set_high();

		//check if the radio responds by seeing if we can change a register
		let mut synced = false;
		for _attempt in 0..100 {
			self.write_reg(Register::Syncvalue1, 0xAA); //170
			self.delay.delay_ms(1);
			if self.read_reg(Register::Syncvalue1) == 0xAA {
				synced = true;
				break;
			}
		}
		if !synced {return Err("could not communicate with radio")}

		synced =	false;
		for _attempt in 0..100 {
			self.write_reg(Register::Syncvalue1, 0x55); //85
			self.delay.delay_ms(1);
			if self.read_reg(Register::Syncvalue1) == 0x55 {
				synced = true;
				break;
			}
		}
		if !synced {return Err("could not communicate with radio")}

		//configure the radio chips for normal use
		self.configure_radio()?;

		Ok(())
	}

	// To enable encryption: radio.encrypt("ABCDEFGHIJKLMNOP");
	// To disable encryption: radio.encrypt(null) or radio.encrypt(0)
	// KEY HAS TO BE 16 bytes !!!
	fn set_encryption_key(&mut self) -> Result<(),&'static str> {

		self.switch_transeiver_mode_blocking(RadioMode::Standby)?;

		match self.encryption_key {
			None =>
				self.register_flags.config2 &= !registers::PacketConfig2::Aes_On, //set aes off
			Some(mut key) => {
		  	self.register_flags.config2 |= registers::PacketConfig2::Aes_On; //set aes on
		  	key[0] = Register::Aeskey1.write_address();
		  	self.spi.write(&key).unwrap();
			},
		}
		self.delay.delay_us(15u16);

		self.write_reg(Register::Packetconfig2, self.register_flags.config2.bits());
		self.switch_transeiver_mode_blocking(RadioMode::Rx)?;
		Ok(())
	}

	fn set_power_level(&mut self) {
		use crate::registers::PaLevel;
		self.register_flags.pa_level -= PaLevel::Power;
		self.register_flags.pa_level |= PaLevel::from_bits(self.power_level).unwrap_or(PaLevel::Power);

		self.write_reg(Register::Palevel, self.register_flags.pa_level.bits());
	}

	fn await_interrupt_flag(&mut self, register: Register, flag: registers::IrqFlags2) -> Result<(),&'static str> {
		for _attempt in 0..10 {//try for one millisecond
			let interrupt_flag = registers::IrqFlags2::from_bits(self.read_reg(register)).unwrap();
			if interrupt_flag.contains(flag){
				return Ok(())
			}
			self.delay.delay_us(100u16);
		}
		Err("interrupt flag was not set within timeout")
	}

	pub fn send_blocking(&mut self, adress: u8, buffer: &[u8]) -> Result<(),&'static str> {
		use crate::registers::DioMapping1;

		self.switch_transeiver_mode_blocking(RadioMode::Standby)?;
		//setup the interrupt pin so an interrupt wil fire once the packet has been send
		self.write_reg(Register::Diomapping1, DioMapping1::Dio0_00.bits()); //in tx mode Dio0_00: packet sent

		let return_adress = match self.adress_filtering {
			AddressFiltering::None => {
				0
			},
			AddressFiltering::AddressOnly(node_addr) => {
				node_addr
			},
			AddressFiltering::AddressOrBroadcast((node_addr,_broadcast_addr)) => {
				node_addr
			},
		};

		//spiXfer(spi_handle,  (char*)rawDATA, (char*)rawDATA, bufferSize + 5 );
		let mut packet = [0u8; registers::MAX_PACKET_SIZE+3];
		let send_len = min(buffer.len() + 3, registers::MAX_PACKET_SIZE);
		packet[0] = Register::Fifo.write_address();
		packet[1] = send_len as u8;
		packet[2] = adress; //1
		packet[3] = return_adress; //2
		packet[4] = 0;//reserved;  //3

		packet[5..5+buffer.len()].clone_from_slice(buffer);

		//self.cs.set_low();
		self.spi.write(&packet[..5+buffer.len()]).unwrap();
		//self.cs.set_high();

		self.delay.delay_us(15u16);

		// no need to wait for transmit mode to be ready since its handled by the radio
		self.switch_transeiver_mode_blocking(RadioMode::Tx)?;

		self.await_interrupt_flag(Register::Irqflags2, registers::IrqFlags2::Packetsent)?;

		self.switch_transeiver_mode_blocking(RadioMode::Rx)?;
		Ok(())
	}

	fn set_payload_length(&mut self){
		match self.package_len {
			PackageLength::Fixed(len) => {
				self.register_flags.config1 -= registers::PacketConfig1::Format_Variable;
				self.write_reg(Register::Payloadlength, len);
			},
			PackageLength::Max(len) => {
				self.register_flags.config1 |= registers::PacketConfig1::Format_Variable;
				self.write_reg(Register::Payloadlength, len);
			},
		}
		self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
	}

	fn set_default_config(&mut self) {
		for (register, bitflag) in registers::DEFAULT_RADIO_CONFIG.iter() {
			self.write_reg(*register, *bitflag);
		}
	}

	fn set_package_filtering(&mut self) {
		use registers::SyncConfig;
		use registers::PacketConfig1;

		match self.network_filtering {
			None =>	{//switch to one sync word (second one is used as network id)
				self.register_flags.sync = (self.register_flags.sync - SyncConfig::Size) | SyncConfig::Size_1;
				self.write_reg(Register::Syncconfig, self.register_flags.sync.bits());
			},
			Some(network_id) => {
				self.register_flags.sync = (self.register_flags.sync - SyncConfig::Size) | SyncConfig::Size_2;
				self.write_reg(Register::Syncconfig, self.register_flags.sync.bits());
				self.write_reg(Register::Syncvalue2, network_id.get());
			},
		}

		self.register_flags.config1 -= PacketConfig1::Adrsfiltering;
		match self.adress_filtering {
			AddressFiltering::None => {
				self.register_flags.config1 |= PacketConfig1::Adrsfiltering_Off;
				self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
			},
			AddressFiltering::AddressOnly(node_addr) => {
				self.register_flags.config1 |= PacketConfig1::Adrsfiltering_Node;
				self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
				self.write_reg(Register::Nodeadrs, node_addr);
			},
			AddressFiltering::AddressOrBroadcast((node_addr,broadcast_addr)) => {
				self.register_flags.config1 |= PacketConfig1::Adrsfiltering_Nodebroadcast;
				self.write_reg(Register::Packetconfig1, self.register_flags.config1.bits());
				self.write_reg(Register::Nodeadrs, node_addr);
				self.write_reg(Register::Broadcastadrs, broadcast_addr);
			},
		}
	}



	fn set_bitrate(&mut self) {
		//bitrate reg value: F_xosc / bitrate (b/s)
		match self.bitrate {
			Bitrate::Lowest => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_1200.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_1200.bits());
			},
			Bitrate::Low => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_55555.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_55555.bits());
			},
			Bitrate::High => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_200kbps.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_200kbps.bits());
			},
			Bitrate::Standard => {
				self.write_reg(Register::Bitratemsb, registers::Bitrate::Msb_100000.bits());
				self.write_reg(Register::Bitratelsb, registers::Bitrate::Lsb_100000.bits());
			},
			Bitrate::Custom(bitrate) => {
				let msb = (FXOSC/bitrate >> 8) as u8;
				let lsb = (FXOSC/bitrate) as u8;
				self.write_reg(Register::Bitratemsb, msb);
				self.write_reg(Register::Bitratelsb, lsb);
			},
		}
	}


	fn switch_freq(&mut self) -> Result<(),&'static str> {
		let frf = (self.freq as f32 / RF69_FSTEP) as u32; // divide down by FSTEP to get FRF
		if self.mode == RadioMode::Tx {
			self.switch_transeiver_mode_blocking(RadioMode::Rx)?;
			self.write_reg(Register::Frfmsb, (frf >> 16) as u8);
			self.write_reg(Register::Frfmid, (frf >> 8) as u8);
			self.write_reg(Register::Frflsb, frf as u8);
			self.switch_transeiver_mode_blocking(RadioMode::Tx)?;
		} else {
			let old_mode = self.mode;
			self.write_reg(Register::Frfmsb, (frf >> 16) as u8);
			self.write_reg(Register::Frfmid, (frf >> 8) as u8);
			self.write_reg(Register::Frflsb, frf as u8);
			self.switch_transeiver_mode_blocking(RadioMode::FreqSynth)?;
			self.switch_transeiver_mode_blocking(old_mode)?;
		}
		Ok(())
	}

	//see page 38 in the datasheet,
	//TODO research Fdev and do that too
	fn set_frequency(&mut self) -> Result<(),&'static str> {
	  if !self.register_flags.mode.contains(registers::OpMode::Sequencer_Off) {
	  	self.register_flags.mode |= registers::OpMode::Sequencer_Off;
	  	self.write_reg(Register::Opmode, self.register_flags.mode.bits());

	  	self.switch_freq()?;

	  	self.register_flags.mode -= registers::OpMode::Sequencer_Off;
	  	self.write_reg(Register::Opmode, self.register_flags.mode.bits());
	  } else {
	  	self.switch_freq()?;
	  }
		Ok(())
	}

	fn switch_transceiver_mode(&mut self, new_mode: RadioMode) {
		use registers::OpMode;

		let old_flag = self.register_flags.mode - OpMode::Mode;
		self.register_flags.mode = match new_mode {
			RadioMode::Sleep => old_flag | OpMode::Sleep, // Xtal Off
			RadioMode::Standby => old_flag | OpMode::Standby, // Xtal On
			RadioMode::FreqSynth => old_flag | OpMode::Synthesizer, // Pll On
			RadioMode::Rx => old_flag | OpMode::Receiver, // Rx Mode
			RadioMode::Tx => old_flag | OpMode::Transmitter, // Tx Mode
		};
		self.write_reg(Register::Opmode, self.register_flags.mode.bits());
		self.mode = new_mode;
	}

	fn switch_transeiver_mode_blocking(&mut self, new_mode: RadioMode) -> Result<(),&'static str>{
		use registers::IrqFlags1;

		self.switch_transceiver_mode(new_mode);
		for _attempt in 0..10 {//try for one millisecond
			let interrupt_flag = IrqFlags1::from_bits(self.read_reg(Register::Irqflags1)).unwrap();
			if interrupt_flag.contains(IrqFlags1::Modeready){
				return Ok(())
			}
			self.delay.delay_us(100u16);
		}
		Err("transiever did not switch within timeout")
	}

	fn write_reg(&mut self, addr: Register, value: u8) {
		let to_write: [u8; 2] = [addr.write_address(), value];

		//self.cs.set_low();
		self.spi.write(&to_write).unwrap();
		//self.cs.set_high();
		self.delay.delay_us(15u16);
	}

	fn read_reg(&mut self, addr: Register) -> u8{
		let mut to_transfer: [u8; 2] = [addr.read_address(), 0];

		//self.cs.set_low();
		let to_transfer = self.spi.transfer(&mut to_transfer).unwrap();
		//self.cs.set_high();
		self.delay.delay_us(15u16);

		let awnser = to_transfer[1];
		awnser
	}

}

