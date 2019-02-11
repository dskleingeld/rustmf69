#include "rfm69.h"
//#include <stdio.h>
#include <stdint.h>
#include <iostream>

#include <pigpio.h>

#define NODEID      99
#define NETWORKID   100
#define GATEWAYID   1
#define FREQUENCY   RF69_433MHZ //Match this with the version of your Moteino! (others: RF69_433MHZ, RF69_868MHZ)
#define KEY         "sampleEncryptKey" //has to be same 16 characters/bytes on all nodes, not more not less!
#define ACK_TIME    30  // # of ms to wait for an ack

#include <chrono>

constexpr int slaveSelectPin = 7;
constexpr int interruptPin = 25;


RFM69 radio;

int main() {
	if(!radio.initialize(FREQUENCY,NODEID,NETWORKID, low))
		std::cout<<"COULD NOT INIT RADIO"<<std::endl;
	radio.encrypt(KEY);
	radio.setPowerLevel(0); //0-31
	//radio.setFrequency(433000000);
	
	char sendBuf1[2] = {'h','i'};
	char sendBuf2[2] = {'g','i'};
	
  std::chrono::steady_clock::time_point begin = std::chrono::steady_clock::now();
	int timeOuts = 0;
	for(int i = 0; i<100; i++){
		//radio.sendWithRetry(98, sendBuf1, 2, 5,40);
		//radio.recieveFor(4000);
		//std::cout<<"raw buffer: ";
		//for(uint8_t i = 3; i< radio.PAYLOADLEN+3; i++)
		//	std::cout<<*(radio.data+i);
		//std::cout<<std::endl;
		//radio.setDataRead();
		
		//radio.sendWithRetry(98, sendBuf2, 2, 5,40);
		//radio.recieveFor(4000);
		//radio.setDataRead();
		std::cout<<"#ssssssss = "<<std::endl;
		radio.send(98, sendBuf2, 2);
		std::cout<<"#ssssssss = "<<std::endl;
		if(radio.recieveFor(40000))
			radio.setDataRead();
		else
			timeOuts++;
		std::cout<<i<<std::endl;
	}
	std::cout<<"#timeOuts = " << timeOuts <<std::endl;
	
	std::chrono::steady_clock::time_point end= std::chrono::steady_clock::now();
	
  std::cout << "Time difference = " << std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count() <<std::endl;

	
	
	return 0;
}