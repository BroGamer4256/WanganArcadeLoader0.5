#!/bin/bash
export LD_LIBRARY_PATH="${PWD};${PWD}/libso"
export LD_PRELOAD="libwal_3dxp.so"
export MANGOHUD_CONFIG="fps_limit=60;no_display=1"
mangohud --dlsym ./main
