// **********************************************************************************
// Driver definition for HopeRF RFM69W/RFM69HW/RFM69CW/RFM69HCW, Semtech SX1231/1231H
// **********************************************************************************
// Copyright Felix Rusu (2014), felix@lowpowerlab.com
// http://lowpowerlab.com/
// Raspberry Pi port by Alexandre Bouillot (2014-2015) @abouillot on twitter
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
#ifndef RFM69_h
#define RFM69_h

#include <stdint.h>
#include <sys/time.h>
#include <ctime>
#include <pigpio.h>
#include <thread>         // std::this_thread::sleep_for
#include <chrono>         // std::chrono::seconds
#include <cstring>

#define RF69_MAX_DATA_LEN     61 // to take advantage of the built in AES/CRC we want to limit the frame size to the internal FIFO size (66 bytes - 3 bytes overhead - 2 bytes crc)

#define RF69_IRQ_PIN          6
#define RF69_IRQ_NUM          0
#define SPI_SPEED 500000
#define SPI_DEVICE 0

#define CSMA_LIMIT              -90 // upper RX signal sensitivity threshold in dBm for carrier sense access
#define RF69_MODE_SLEEP         0 // XTAL OFF
#define RF69_MODE_STANDBY       1 // XTAL ON
#define RF69_MODE_SYNTH         2 // PLL ON
#define RF69_MODE_RX            3 // RX MODE
#define RF69_MODE_TX            4 // TX MODE

// available frequency bands
#define RF69_315MHZ            31 // non trivial values to avoid misconfiguration
#define RF69_433MHZ            43
#define RF69_868MHZ            86
#define RF69_915MHZ            91

#define null                  0
#define COURSE_TEMP_COEF    -90 // puts the temperature reading in the ballpark, user can fine tune the returned value
#define RF69_BROADCAST_ADDR 255
#define RF69_CSMA_LIMIT_MS 10000
#define RF69_TX_LIMIT_MS   10000
#define RF69_FSTEP  61.03515625 // == FXOSC / 2^19 = 32MHz / 2^19 (p13 in datasheet)

// TWS: define CTLbyte bits
#define RFM69_CTL_SENDACK   0x80
#define RFM69_CTL_REQACK    0x40


enum Bitrate { low, high, standard };

class RFM69 {
  public:
		uint8_t PAYLOADLEN;
		uint8_t* data;
		
    RFM69() {
      _mode = RF69_MODE_RX;
      _powerLevel = 31;
			data = (uint8_t*)&rawDATA[3];
			rawDATA[rawDATALEN] = 0; // add null at end of string
    }

    bool initialize(uint8_t freqBand, uint8_t ID, uint8_t networkID=1, Bitrate speed=low);
    bool canSend();
    virtual void send(uint8_t toAddress, const void* buffer, uint8_t bufferSize, bool requestACK=false);
    virtual bool sendWithRetry(uint8_t toAddress, const void* buffer, uint8_t bufferSize, uint8_t retries=2, uint8_t retryWaitTime=40); 
		// 40ms roundtrip req for 61byte packets
    bool ACKReceived(uint8_t fromNodeID);
    bool ACKRequested();
    virtual void sendACK(const void* buffer = "", uint8_t bufferSize=0);
    void setFrequency(uint32_t freqHz);
    void encrypt(const char* key);
    int16_t readRSSI(bool forceTrigger=false);
    virtual void setPowerLevel(uint8_t level); // reduce/increase transmit power level
    uint8_t readTemperature(uint8_t calFactor=0); // get CMOS temperature (8bit)
    void rcCalibration(); // calibrate the internal RC oscillator for use in wide temperature variations - see datasheet section [4.3.5. RC Timer Accuracy]
		void recieve();
		bool recieveFor(uint16_t timeOut);
		void setDataRead(){
			PAYLOADLEN = 0;
		}
		
    // allow hacking registers by making these public
    uint8_t readReg(uint8_t addr);
    void writeReg(uint8_t addr, uint8_t val);
    void readAllRegs();

  protected:
	  uint8_t rawDATA[RF69_MAX_DATA_LEN]; // recv/xmit buf, including header & crc bytes
    uint8_t rawDATALEN;
    uint8_t SENDERID;
    uint8_t TARGETID; // should match _address
    uint8_t ACK_REQUESTED;
    uint8_t ACK_RECEIVED; // should be polled immediately after sending a packet with ACK request
    int16_t RSSI; // most accurate RSSI during reception (closest to the reception)
    uint8_t _mode; // should be protected?
	
		bool recieveShort();	
    virtual void sendFrame(uint8_t toAddress, const void* buffer, uint8_t size, bool requestACK=false, bool sendACK=false);
		bool packageInFifo();
		
    //uint8_t _address;
		uint8_t _address;
    uint8_t _powerLevel;
    uint8_t _SPCR;
    uint8_t _SPSR;
    uint8_t spi_handle;
		bool gotPayloadInsteadOfAwk = false;
		
    virtual void setMode(uint8_t mode);
		uint32_t timeMicroSec();
		void delayMicroseconds(int);
};

#endif
