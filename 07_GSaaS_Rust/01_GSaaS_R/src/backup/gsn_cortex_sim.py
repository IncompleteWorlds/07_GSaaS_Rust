#!/usr/bin/env python
#

#
# Send TCP messages read from a file
#
import sys
import time
from socket import *

HOST = 'localhost'
PORT = 20021
FILE_NAME = 'data_file.bin'
BUFFER_SIZE = 65536
FRAME_LENGTH = 1115
ADDR = (HOST, PORT)
DATA_PORT = 20022
# Delay between frames in milliseconds
DELAY_BETWEEN_FRAME=1000


def checkArgs():
    argc = len(sys.argv)
    if (argc < 3):
        print("ERROR: Incorrect number of arguments")
        print("Usage: ")
        print("       gsn_cortex_sim    ip_address  ip_port    binary_data_file   [frame_length]")
        print("") 
        sys.exit(-1)

    if (argc > 3):
        FRAME_LENGTH = sys.argv[4]

    HOST = sys.argv[1]
    PORT = sys.argv[2]
    FILE_NAME = sys.argv[3]

    print("Host = ", HOST)
    print("Port = ", PORT)
    print("Data file = ", FILE_NAME)
        

def read_frame():
    pass

def mainFunction():
    # TCP socket
    tcpCliSock = socket(AF_INET, SOCK_STREAM)
    tcpCliSock.connect(ADDR)

    while 1:
        data = read_frame()
        if not data:
            break

        tcpCliSock.send(data)

        data = tcpCliSock.recv(BUFFER_SIZE)

        if not data:
            break

        print(data)

        # Wait for delay
        # delay(DELAY_BETWEEN_FRAME)

    tcpCliSock.close()


#
# Main
#
if __name__ == '__main__':
    checkArgs()
    
    mainFunction()

    sys.exit(0)

    

