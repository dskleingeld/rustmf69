// **********************************************************************************
// Driver definition for HopeRF RFM69W/RFM69HW/RFM69CW/RFM69HCW, Semtech SX1231/1231H
// **********************************************************************************
// Copyright Felix Rusu (2014), felix@lowpowerlab.com
// http://lowpowerlab.com/
// Raspberry Pi port by Alexandre Bouillot (2014) @abouillot on twitter
// **********************************************************************************
// License
// **********************************************************************************
// This program is free software; you can redistribute it 
// and/or modify it under the terms of the GNU General    
// Public License as published by the Free Software       
// Foundation; either version 3 of the License, or        
// (at your option) any later version.                    
//                                                        
// This program is distributed in the hope that it will   
// be useful, but WITHOUT ANY WARRANTY; without even the  
// implied warranty of MERCHANTABILITY or FITNESS FOR A   
// PARTICULAR PURPOSE. See the GNU General Public        
// License for more details.                              
//                                                        
// You should have received a copy of the GNU General    
// Public License along with this program.
// If not, see <http://www.gnu.org/licenses/>.
//                                                        
// Licence can be viewed at                               
// http://www.gnu.org/licenses/gpl-3.0.txt
//
// Please maintain this license information along with authorship
// and copyright notices in any redistribution of this code
// **********************************************************************************
#include "rfm69.h"
#include "rfm69registers.h"
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
#include <iostream>
#include <unistd.h>	//usleep
#define MICROSLEEP_LENGTH 15

uint16_t intCount = 0;

bool RFM69::initialize(uint8_t freqBand, uint8_t nodeID, uint8_t networkID, Bitrate speed) {
  const uint8_t CONFIG[][2] =
  {
    /* 0x01 */ { REG_OPMODE, RF_OPMODE_SEQUENCER_ON | RF_OPMODE_LISTEN_OFF | RF_OPMODE_STANDBY },
    /* 0x02 */ { REG_DATAMODUL, RF_DATAMODUL_DATAMODE_PACKET | RF_DATAMODUL_MODULATIONTYPE_FSK | RF_DATAMODUL_MODULATIONSHAPING_00 }, // no shaping
    /* 0x03 */ { REG_BITRATEMSB, RF_BITRATEMSB_55555}, // default: 4.8 KBPS
    /* 0x04 */ { REG_BITRATELSB, RF_BITRATELSB_55555},
    /* 0x05 */ { REG_FDEVMSB, RF_FDEVMSB_50000}, // default: 5KHz, (FDEV + BitRate / 2 <= 500KHz)
    /* 0x06 */ { REG_FDEVLSB, RF_FDEVLSB_50000},

    /* 0x07 */ { REG_FRFMSB, (uint8_t) (freqBand==RF69_315MHZ ? RF_FRFMSB_315 : (freqBand==RF69_433MHZ ? RF_FRFMSB_433 : (freqBand==RF69_868MHZ ? RF_FRFMSB_868 : RF_FRFMSB_915))) },
    /* 0x08 */ { REG_FRFMID, (uint8_t) (freqBand==RF69_315MHZ ? RF_FRFMID_315 : (freqBand==RF69_433MHZ ? RF_FRFMID_433 : (freqBand==RF69_868MHZ ? RF_FRFMID_868 : RF_FRFMID_915))) },
    /* 0x09 */ { REG_FRFLSB, (uint8_t) (freqBand==RF69_315MHZ ? RF_FRFLSB_315 : (freqBand==RF69_433MHZ ? RF_FRFLSB_433 : (freqBand==RF69_868MHZ ? RF_FRFLSB_868 : RF_FRFLSB_915))) },

    // looks like PA1 and PA2 are not implemented on RFM69W, hence the max output power is 13dBm
    // +17dBm and +20dBm are possible on RFM69HW
    // +13dBm formula: Pout = -18 + OutputPower (with PA0 or PA1**)
    // +17dBm formula: Pout = -14 + OutputPower (with PA1 and PA2)**
    // +20dBm formula: Pout = -11 + OutputPower (with PA1 and PA2)** and high power PA settings (section 3.3.7 in datasheet)
    ///* 0x11 */ { REG_PALEVEL, RF_PALEVEL_PA0_ON | RF_PALEVEL_PA1_OFF | RF_PALEVEL_PA2_OFF | RF_PALEVEL_OUTPUTPOWER_11111},
    ///* 0x13 */ { REG_OCP, RF_OCP_ON | RF_OCP_TRIM_95 }, // over current protection (default is 95mA)

    // RXBW defaults are { REG_RXBW, RF_RXBW_DCCFREQ_010 | RF_RXBW_MANT_24 | RF_RXBW_EXP_5} (RxBw: 10.4KHz)
    /* 0x19 */ { REG_RXBW, RF_RXBW_DCCFREQ_010 | RF_RXBW_MANT_16 | RF_RXBW_EXP_2 }, // (BitRate < 2 * RxBw)
    //for BR-19200: /* 0x19 */ { REG_RXBW, RF_RXBW_DCCFREQ_010 | RF_RXBW_MANT_24 | RF_RXBW_EXP_3 },
    /* 0x25 */ { REG_DIOMAPPING1, RF_DIOMAPPING1_DIO0_01 }, // DIO0 is the only IRQ we're using
    /* 0x26 */ { REG_DIOMAPPING2, RF_DIOMAPPING2_CLKOUT_OFF }, // DIO5 ClkOut disable for power saving
    /* 0x28 */ { REG_IRQFLAGS2, RF_IRQFLAGS2_FIFOOVERRUN }, // writing to this bit ensures that the FIFO & status flags are reset
    /* 0x29 */ { REG_RSSITHRESH, 220 }, // must be set to dBm = (-Sensitivity / 2), default is 0xE4 = 228 so -114dBm
    ///* 0x2D */ { REG_PREAMBLELSB, RF_PREAMBLESIZE_LSB_VALUE } // default 3 preamble bytes 0xAAAAAA
    /* 0x2E */ { REG_SYNCCONFIG, RF_SYNC_ON | RF_SYNC_FIFOFILL_AUTO | RF_SYNC_SIZE_2 | RF_SYNC_TOL_0 },
    /* 0x2F */ { REG_SYNCVALUE1, 0x2D },      // attempt to make this compatible with sync1 byte of RFM12B lib
    /* 0x30 */ { REG_SYNCVALUE2, networkID }, // NETWORK ID
    /* 0x37 */ { REG_PACKETCONFIG1, RF_PACKET1_FORMAT_VARIABLE | RF_PACKET1_DCFREE_OFF | RF_PACKET1_CRC_ON | RF_PACKET1_CRCAUTOCLEAR_ON | RF_PACKET1_ADRSFILTERING_OFF },
    /* 0x38 */ { REG_PAYLOADLENGTH, 66 }, // in variable length mode: the max frame size, not used in TX
    /* 0x39 */ { REG_NODEADRS, nodeID }, //  address filtering
    /* 0x3C */ { REG_FIFOTHRESH, RF_FIFOTHRESH_TXSTART_FIFONOTEMPTY | RF_FIFOTHRESH_VALUE }, // TX on FIFO not empty
    /* 0x3D */ { REG_PACKETCONFIG2, RF_PACKET2_RXRESTARTDELAY_2BITS | RF_PACKET2_AUTORXRESTART_ON | RF_PACKET2_AES_OFF }, // RXRESTARTDELAY must match transmitter PA ramp-down time (bitrate dependent)
    //for BR-19200: /* 0x3D */ { REG_PACKETCONFIG2, RF_PACKET2_RXRESTARTDELAY_NONE | RF_PACKET2_AUTORXRESTART_ON | RF_PACKET2_AES_OFF }, // RXRESTARTDELAY must match transmitter PA ramp-down time (bitrate dependent)
    /* 0x6F */ { REG_TESTDAGC, RF_DAGC_IMPROVED_LOWBETA0 }, // run DAGC continuously in RX mode for Fading Margin Improvement, recommended default for AfcLowBetaOn=0
    {255, 0}
  };

  // Initialize SPI device 0
  if (!gpioInitialise()) {
    printf("gpio init failed. Are you running as root??\n");
	return false;
  }
  
	/*
  21 20 19 18 17 16 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
   b  b  b  b  b  b  R  T  n  n  n  n  W  A u2 u1 u0 p2 p1 p0  m  m
   0  0  0  0  0  0  0  0  0  0  0  0  0  0  1  0  1  0  0  0  0  0
  
  mm defines the SPI mode. 
	Warning: modes 1 and 3 do not appear to work on the auxiliary SPI. 
	Mode POL PHA
	 0    0   0
	 1    0   1
	 2    1   0
	 3    1   1

	px is 0 if CEx is active low (default) and 1 for active high. 
	ux is 0 if the CEx GPIO is reserved for SPI (default) and 1 otherwise. 
	A is 0 for the main SPI, 1 for the auxiliary SPI. 
	W is 0 if the device is not 3-wire, 1 if the device is 3-wire. Main SPI only. 
	nnnn defines the number of bytes (0-15) to write before switching the MOSI line to MISO to read data. This field is ignored if W is not set. Main SPI only. 
	T is 1 if the least significant bit is transmitted on MOSI first, the default (0) shifts the most significant bit out first. Auxiliary SPI only. 
	R is 1 if the least significant bit is received on MISO first, the default (0) receives the most significant bit first. Auxiliary SPI only. 
	bbbbbb defines the word size in bits (0-32). The default (0) sets 8 bits per word. Auxiliary SPI only. 
  */

  constexpr uint32_t spiFlags = 0 | (1 << 7)
	                      | (1 << 5);
    constexpr int spiChan = 1; 											
  constexpr int spi_speed = 500000; 
	if ((spi_handle = spiOpen(spiChan, spi_speed, spiFlags)) < 0)
		printf("spi failed.\n");
	
  uint32_t start_t = timeMicroSec();
  uint32_t timeout = 80000;
  do writeReg(REG_SYNCVALUE1, 0xAA); while (readReg(REG_SYNCVALUE1) != 0xAA && timeMicroSec()-start_t < timeout);
  start_t = timeMicroSec();
  do writeReg(REG_SYNCVALUE1, 0x55); while (readReg(REG_SYNCVALUE1) != 0x55 && timeMicroSec()-start_t < timeout);

  for (uint8_t i = 0; CONFIG[i][0] != 255; i++)
    writeReg(CONFIG[i][0], CONFIG[i][1]);

	switch(speed){
		case low:
			writeReg(REG_BITRATEMSB, RF_BITRATEMSB_55555); // default: 4.8 KBPS
			writeReg(REG_BITRATELSB, RF_BITRATELSB_55555);
			break;
		case standard:
			writeReg(REG_BITRATEMSB, RF_BITRATEMSB_100000); // default: 4.8 KBPS
			writeReg(REG_BITRATELSB, RF_BITRATELSB_100000);
			break;
		case high:
			writeReg(REG_BITRATEMSB, RF_BITRATEMSB_200KBPS); // default: 4.8 KBPS
			writeReg(REG_BITRATELSB, RF_BITRATELSB_200KBPS);
			break;
	}
	
  // Encryption is persistent between resets and can trip you up during debugging.
  // Disable it during initialization so we always start from a known state.
  encrypt(0);

  setMode(RF69_MODE_STANDBY);
  start_t = timeMicroSec();
  while (((readReg(REG_IRQFLAGS1) & RF_IRQFLAGS1_MODEREADY) == 0x00) && timeMicroSec()-start_t < timeout); // wait for ModeReady
  if (timeMicroSec()-start_t >= timeout){
		std::cout<<"gggrrrr"<<+readReg(REG_IRQFLAGS1)<<std::endl;
    return false;
  }
  _address = nodeID;
	setMode(RF69_MODE_RX);
  return true;
}

// set the frequency (in Hz)
void RFM69::setFrequency(uint32_t freqHz) {
  uint8_t oldMode = _mode;
  if (oldMode == RF69_MODE_TX) {
    setMode(RF69_MODE_RX);
  }
  freqHz /= RF69_FSTEP; // divide down by FSTEP to get FRF
  writeReg(REG_FRFMSB, freqHz >> 16);
  writeReg(REG_FRFMID, freqHz >> 8);
  writeReg(REG_FRFLSB, freqHz);
  if (oldMode == RF69_MODE_RX) {
    setMode(RF69_MODE_SYNTH);
  }
  setMode(oldMode);
}

void RFM69::setMode(uint8_t newMode) {
  if (newMode == _mode)
    return;

  switch (newMode) {
    case RF69_MODE_TX:
      writeReg(REG_OPMODE, (readReg(REG_OPMODE) & 0xE3) | RF_OPMODE_TRANSMITTER);
      break;
    case RF69_MODE_RX:
      writeReg(REG_OPMODE, (readReg(REG_OPMODE) & 0xE3) | RF_OPMODE_RECEIVER);
      break;
    case RF69_MODE_SYNTH:
      writeReg(REG_OPMODE, (readReg(REG_OPMODE) & 0xE3) | RF_OPMODE_SYNTHESIZER);
      break;
    case RF69_MODE_STANDBY:
      writeReg(REG_OPMODE, (readReg(REG_OPMODE) & 0xE3) | RF_OPMODE_STANDBY);
      break;
    case RF69_MODE_SLEEP:
      writeReg(REG_OPMODE, (readReg(REG_OPMODE) & 0xE3) | RF_OPMODE_SLEEP);
      break;
    default:
      return;
  }
  _mode = newMode;
}

// set *transmit/TX* output power: 0=min, 31=max
// this results in a "weaker" transmitted signal, and directly results in a lower RSSI at the receiver
// the power configurations are explained in the SX1231H datasheet (Table 10 on p21; RegPaLevel p66): http://www.semtech.com/images/datasheet/sx1231h.pdf
// valid powerLevel parameter values are 0-31 and result in a directly proportional effect on the output/transmission power
// this function implements 2 modes as follows:
//       - for RFM69W the range is from 0-31 [-18dBm to 13dBm] (PA0 only on RFIO pin)
//       - for RFM69HW the range is from 0-31 [5dBm to 20dBm]  (PA1 & PA2 on PA_BOOST pin & high Power PA settings - see section 3.3.7 in datasheet, p22)
void RFM69::setPowerLevel(uint8_t powerLevel) {
  _powerLevel = (powerLevel > 31 ? 31 : powerLevel);
  writeReg(REG_PALEVEL, (readReg(REG_PALEVEL) & 0xE0) | _powerLevel);
}

bool RFM69::canSend() {
  if (_mode == RF69_MODE_RX and PAYLOADLEN == 0 and readRSSI() < CSMA_LIMIT) { // if signal stronger than -100dBm is detected assume channel activity
    setMode(RF69_MODE_STANDBY);
		return true;
  }
	printf("RF69_MODE_RX %d RF69_MODE_STANDBY %d\n", RF69_MODE_RX, RF69_MODE_STANDBY);
  printf("mode %d, payload %d, rssi %d %d\n", _mode, PAYLOADLEN, readRSSI(), CSMA_LIMIT);
  return false;
}

void RFM69::send(uint8_t toAddress, const void* buffer, uint8_t bufferSize, bool requestACK) {

  writeReg(REG_PACKETCONFIG2, (readReg(REG_PACKETCONFIG2) & 0xFB) | RF_PACKET2_RXRESTART); // avoid RX deadlocks
  while (!canSend());
	std::cout<<"#ssssssss = "<<std::endl;
	sendFrame(toAddress, buffer, bufferSize, requestACK, false);
	}

// to increase the chance of getting a packet across, call this function instead of send
// and it handles all the ACK requesting/retrying for you :)
// The only twist is that you have to manually listen to ACK requests on the other side and send back the ACKs
// The reason for the semi-automaton is that the lib is interrupt driven and
// requires user action to read the received data and decide what to do with it
// replies usually take only 5..8ms at 50kbps@915MHz
bool RFM69::sendWithRetry(uint8_t toAddress, const void* buffer, uint8_t bufferSize, uint8_t retries, uint8_t retryWaitTime) {
  uint32_t sentTime;
  for (uint8_t i = 0; i <= retries; i++) {
    send(toAddress, buffer, bufferSize, true);
    sentTime = timeMicroSec();
    while (timeMicroSec() - sentTime < retryWaitTime*1000) {
      if (packageInFifo()) {
				ACKReceived(toAddress);
        printf(" ~us: %d\n", timeMicroSec() - sentTime);
				return true;
      }
    }
    printf(" RETRY# %d\n", i + 1);
  }
	return false;
}

// should be polled immediately after sending a packet with ACK request
bool RFM69::ACKReceived(uint8_t fromNodeID) {
	if(recieveShort()){
		if (PAYLOADLEN > 0){
			if(!ACK_RECEIVED){
				std::cout<<"..................."<<std::endl;
				gotPayloadInsteadOfAwk = true;
				return true;
			}
			return (SENDERID == fromNodeID || fromNodeID == RF69_BROADCAST_ADDR) && ACK_RECEIVED;
		}
	}
  return false;
}

// check whether an ACK was requested in the last received packet (non-broadcasted packet)
bool RFM69::ACKRequested() {
  return ACK_REQUESTED && (TARGETID != RF69_BROADCAST_ADDR);
}

// should be called immediately after reception in case sender wants ACK
void RFM69::sendACK(const void* buffer, uint8_t bufferSize) {
  ACK_REQUESTED = 0;   // TWS added to make sure we don't end up in a timing race and infinite loop sending Acks
  uint8_t sender = SENDERID;
  int16_t _RSSI = RSSI; // save payload received RSSI value
  writeReg(REG_PACKETCONFIG2, (readReg(REG_PACKETCONFIG2) & 0xFB) | RF_PACKET2_RXRESTART); // avoid RX deadlocks
  uint32_t start_t = timeMicroSec();
  while (!canSend() && timeMicroSec() - start_t < RF69_CSMA_LIMIT_MS) { delayMicroseconds(MICROSLEEP_LENGTH);}
  sendFrame(sender, buffer, bufferSize, false, true);
  RSSI = _RSSI; // restore payload RSSI
}

// internal function
void RFM69::sendFrame(uint8_t toAddress, const void* buffer, uint8_t bufferSize, bool requestACK, bool sendACK) {
  setMode(RF69_MODE_STANDBY); // turn off receiver to prevent reception while filling fifo
  while ((readReg(REG_IRQFLAGS1) & RF_IRQFLAGS1_MODEREADY) == 0x00); // wait for ModeReady
  writeReg(REG_DIOMAPPING1, RF_DIOMAPPING1_DIO0_00); // DIO0 is "Packet Sent"
  if (bufferSize > RF69_MAX_DATA_LEN) bufferSize = RF69_MAX_DATA_LEN;

  // control byte
  uint8_t CTLbyte = 0x00;
  if (sendACK)
    CTLbyte = RFM69_CTL_SENDACK;
  else if (requestACK)
    CTLbyte = RFM69_CTL_REQACK;

  unsigned char rawDATA[63];
  uint8_t i;
  for(i = 0; i < 63; i++) rawDATA[i] = 0;

  rawDATA[0] = REG_FIFO | 0x80;
  rawDATA[1] = bufferSize + 3;
  rawDATA[2] = toAddress;
  rawDATA[3] = _address;
  rawDATA[4] = CTLbyte;

  // write to FIFO
  for(i = 0; i < bufferSize; i++) {
    rawDATA[i + 5] = ((char*)buffer)[i];
  }
  spiXfer(spi_handle,  (char*)rawDATA, (char*)rawDATA, bufferSize + 5 );

  // no need to wait for transmit mode to be ready since its handled by the radio
  setMode(RF69_MODE_TX);
  
  while ((readReg(REG_IRQFLAGS2) & RF_IRQFLAGS2_PACKETSENT) == 0x00); // reg check poll; wait for transmission finish
  setMode(RF69_MODE_RX);
}

bool RFM69::packageInFifo(){
	return (readReg(REG_IRQFLAGS2) & RF_IRQFLAGS2_PAYLOADREADY) != 0;
}

bool RFM69::recieveShort() {
		setMode(RF69_MODE_STANDBY);
	rawDATA[0] = REG_FIFO & 0x7F;
	rawDATA[1] = 0; // PAYLOADLEN
	rawDATA[2] = 0; //  TargetID
	spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, 3 );
	delayMicroseconds(MICROSLEEP_LENGTH);

	PAYLOADLEN = rawDATA[1];
	PAYLOADLEN = PAYLOADLEN > 66 ? 66 : PAYLOADLEN; // precaution
	TARGETID = rawDATA[2];

	rawDATALEN = PAYLOADLEN - 3;
	rawDATA[0] = REG_FIFO & 0x77;
	rawDATA[1] = 0; //SENDERID
	rawDATA[2] = 0; //CTLbyte;
	for(uint8_t i = 0; i< rawDATALEN; i++) {
		rawDATA[i+3] = 0;
	}
	spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, rawDATALEN + 3);
	delayMicroseconds(MICROSLEEP_LENGTH);
	setMode(RF69_MODE_RX);
	RSSI = readRSSI();
	
	SENDERID = rawDATA[1];
	uint8_t CTLbyte = rawDATA[2];

	std::cout<<"awk raw buffer: ";
	for(uint8_t i = 3; i< rawDATALEN+3; i++)
		std::cout<<rawDATA[i];
	std::cout<<std::endl;
	
	
	ACK_RECEIVED = CTLbyte & 0x80; //extract ACK-requested flag
	ACK_REQUESTED = CTLbyte & 0x40; //extract ACK-received flag
	
	return true;
}

// internal function - interrupt gets called when a packet is received
bool RFM69::recieveFor(uint16_t timeOut) {
	uint32_t sentTime = timeMicroSec();
	
	if(gotPayloadInsteadOfAwk){
		gotPayloadInsteadOfAwk = false;
	} else {
		while(!packageInFifo()){
			if(timeMicroSec() - sentTime < timeOut){
				delayMicroseconds(100);
			} else{
				std::cout<<"recieve timeOut"<<std::endl;
				return false;
			}
		}
		setMode(RF69_MODE_STANDBY);
		rawDATA[0] = REG_FIFO & 0x7F;
		rawDATA[1] = 0; // PAYLOADLEN
		rawDATA[2] = 0; //  TargetID
		spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, 3 );
		delayMicroseconds(MICROSLEEP_LENGTH);

		PAYLOADLEN = rawDATA[1];
		PAYLOADLEN = PAYLOADLEN > 66 ? 66 : PAYLOADLEN; // precaution
		TARGETID = rawDATA[2];

		rawDATALEN = PAYLOADLEN - 3;
		rawDATA[0] = REG_FIFO & 0x77;
		rawDATA[1] = 0; //SENDERID
		rawDATA[2] = 0; //CTLbyte;
		std::memset(&rawDATA[3], rawDATALEN, rawDATALEN);
		spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, rawDATALEN + 3);
		setMode(RF69_MODE_RX);
		RSSI = readRSSI();
	}
	
	std::cout<<"recieveFor raw buffer: ";
	for(uint8_t i = 3; i< rawDATALEN+3; i++)
		std::cout<<rawDATA[i];
	std::cout<<std::endl;
  
	
	SENDERID = rawDATA[1];
	uint8_t CTLbyte = rawDATA[2];

	ACK_RECEIVED = CTLbyte & 0x80; //extract ACK-requested flag
	ACK_REQUESTED = CTLbyte & 0x40; //extract ACK-received flag
	return true;
}

// internal function - interrupt gets called when a packet is received
void RFM69::recieve() {
	
  uint8_t i;
  unsigned char rawDATA[67];
	//printf("RF69_MODE_RX %d RF69_MODE_STANDBY %d\n", RF69_MODE_RX, RF69_MODE_STANDBY);
  //printf("mode %d\n", _mode);
	//std::cout<<"sssssssssss"<<std::endl;
	if(gotPayloadInsteadOfAwk)
		gotPayloadInsteadOfAwk = false;
	else
		while((readReg(REG_IRQFLAGS2) & RF_IRQFLAGS2_PAYLOADREADY) == 0)
			delayMicroseconds(1000);
	//std::cout<<"rrrrrrrrrrr"<<std::endl;
	//std::cout<<+( readReg(REG_IRQFLAGS2) & RF_IRQFLAGS2_PAYLOADREADY)<<std::endl;
	
	rawDATA[0] = REG_FIFO & 0x7F;
	rawDATA[1] = 0; // PAYLOADLEN
	rawDATA[2] = 0; //  TargetID
	spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, 3 );
	delayMicroseconds(MICROSLEEP_LENGTH);

	PAYLOADLEN = rawDATA[1];
	PAYLOADLEN = PAYLOADLEN > 66 ? 66 : PAYLOADLEN; // precaution
	TARGETID = rawDATA[2];

	rawDATALEN = PAYLOADLEN - 3;
	rawDATA[0] = REG_FIFO & 0x77;
	rawDATA[1] = 0; //SENDERID
	rawDATA[2] = 0; //CTLbyte;
	for(i = 0; i< rawDATALEN; i++) {
		rawDATA[i+3] = 0;
	}
	spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, rawDATALEN + 3);
	delayMicroseconds(MICROSLEEP_LENGTH);
	
	
	//std::cout<<"raw buffer: ";
	//for(uint8_t i = 3; i< rawDATALEN+3; i++)
	//	std::cout<<rawDATA[i];
	//std::cout<<std::endl;
	
	SENDERID = rawDATA[1];
	uint8_t CTLbyte = rawDATA[2];

	ACK_RECEIVED = CTLbyte & 0x80; //extract ACK-requested flag
	ACK_REQUESTED = CTLbyte & 0x40; //extract ACK-received flag
	if (rawDATALEN < RF69_MAX_DATA_LEN) rawDATA[rawDATALEN] = 0; // add null at end of string
}




// To enable encryption: radio.encrypt("ABCDEFGHIJKLMNOP");
// To disable encryption: radio.encrypt(null) or radio.encrypt(0)
// KEY HAS TO BE 16 bytes !!!
void RFM69::encrypt(const char* key) {
  unsigned char rawDATA[17];
  uint8_t i;

  setMode(RF69_MODE_STANDBY);
  if (key!=0) {
    rawDATA[0] = REG_AESKEY1 | 0x80;
    for(i = 1; i < 17; i++) {
      rawDATA[i] = key[i-1];
    }

    spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, 17);
    delayMicroseconds(MICROSLEEP_LENGTH);
  }

  writeReg(REG_PACKETCONFIG2, (readReg(REG_PACKETCONFIG2) & 0xFE) | (key ? 1 : 0));
	setMode(RF69_MODE_RX);
}

// get the received signal strength indicator (RSSI)
int16_t RFM69::readRSSI(bool forceTrigger) {
  int16_t rssi = 0;
  if (forceTrigger)
  {
    // RSSI trigger not needed if DAGC is in continuous mode
    writeReg(REG_RSSICONFIG, RF_RSSI_START);
    while ((readReg(REG_RSSICONFIG) & RF_RSSI_DONE) == 0x00); // wait for RSSI_Ready
  }
  rssi = -readReg(REG_RSSIVALUE);
  rssi >>= 1;
  return rssi;
}

uint8_t RFM69::readReg(uint8_t addr) {
  uint8_t rawDATA[2];
  rawDATA[0] = addr & 0x7F;
  rawDATA[1] = 0;

  spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, sizeof(rawDATA) );
  delayMicroseconds(MICROSLEEP_LENGTH);

//printf("%x %x\n", addr, rawDATA[1]);
  return rawDATA[1];
}

void RFM69::writeReg(uint8_t addr, uint8_t value) {
//printf("%x %x\n", addr, value);
  uint8_t rawDATA[2];
  rawDATA[0] = addr | 0x80;
  rawDATA[1] = value;

  spiXfer(spi_handle, (char*)rawDATA, (char*)rawDATA, sizeof(rawDATA) );
  delayMicroseconds(MICROSLEEP_LENGTH);
}

// Serial.print all the RFM69 register values
void RFM69::readAllRegs() {
  int i;

  for(i = 1; i <= 0x4F; i++) {
   printf("%i - %i\n\r", i, readReg(i));
  }  
}

uint8_t RFM69::readTemperature(uint8_t calFactor) { // returns centigrade
  setMode(RF69_MODE_STANDBY);
  writeReg(REG_TEMP1, RF_TEMP1_MEAS_START);
  while ((readReg(REG_TEMP1) & RF_TEMP1_MEAS_RUNNING));
  return ~readReg(REG_TEMP2) + COURSE_TEMP_COEF + calFactor; // 'complement' corrects the slope, rising temp = rising val
	setMode(RF69_MODE_RX);
} // COURSE_TEMP_COEF puts reading in the ballpark, user can add additional correction

void RFM69::rcCalibration() {
  writeReg(REG_OSC1, RF_OSC1_RCCAL_START);
  while ((readReg(REG_OSC1) & RF_OSC1_RCCAL_DONE) == 0x00);
}

//only works for half a second (500 millisec) then overflow happens
uint32_t RFM69::timeMicroSec(){
	timeval tv;	
	gettimeofday(&tv, nullptr);
	return tv.tv_usec;
}

void RFM69::delayMicroseconds(int dt){
	std::this_thread::sleep_for (std::chrono::microseconds(dt));
}
