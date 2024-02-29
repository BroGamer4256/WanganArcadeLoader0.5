#!/bin/bash
mkdir -p ./tmp/data/etc/
mkdir -p ./tmp/data/joinstar/
mkdir -p ./tmp/data/maxicoin/
mkdir -p ./tmp/data/ranking/
mkdir -p ./tmp/data/target/
cp ./data/sound/bgm/maxi3/sys_04.wav ./tmp/sys_04.wav

export LD_LIBRARY_PATH="${PWD};${PWD}/libso"
export MANGOHUD_CONFIG="fps_limit=60;no_display=1"
mangohud --dlsym LD_PRELOAD="libwal_3dxp.so" ./main
