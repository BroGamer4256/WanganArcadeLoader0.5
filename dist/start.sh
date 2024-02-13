#!/bin/bash
export LD_LIBRARY_PATH="${PWD};${PWD}/libso"
export LD_PRELOAD="libwal_3dxp.so"
./main
