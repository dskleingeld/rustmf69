MAKEFLAGS+="-j -l $(shell grep -c ^processor /proc/cpuinfo) "

CXX = g++
CXXFLAGS= -Wall -pedantic -std=c++14 #-pedantic


Gateway : Gateway.cpp rfm69.cpp rfm69.h rfm69registers.h
	$(CXX) $(CXXFLAGS) -o Gateway Gateway.cpp rfm69.cpp -pthread -lpigpio -lrt
	


