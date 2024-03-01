#!/bin/bash
mkdir -p ./tmp/data/etc/
mkdir -p ./tmp/data/joinstar/
mkdir -p ./tmp/data/maxicoin/
mkdir -p ./tmp/data/ranking/
mkdir -p ./tmp/data/target/
cp ./data/sound/bgm/maxi3/sys_04.wav ./tmp/sys_04.wav

# Remove old C++ libs
if [ -f ./libso/libstdc++.so.6 ]; then 
	mv ./libso/libstdc++.so.6 ./libso/libstdc++.so.6.bak
fi
if [ -f ./libso/libstdc++.so.6.0.7 ]; then 
	mv ./libso/libstdc++.so.6.0.7 ./libso/libstdc++.so.6.0.7.bak
fi
if [ -f ./libso/libz.so ]; then 
	mv ./libso/libz.so ./libso/libz.so.bak
fi
if [ -f ./libso/libz.so.1 ]; then 
	mv ./libso/libz.so.1 ./libso/libz.so.1.bak
fi

# Recompile shaders for non-nvidia cards
if [ ! -f ./data/shader/.recompiled ]; then
	touch ./data/shader/.recompiled
	for f in ./data/shader/*.cg; do
		file=${f%.cg}
		fp="${file}.fp"
		vp="${file}.vp"
		if [ -f $fp ]; then
			rm $fp
		fi
		if [ -f $vp ]; then
			rm $vp
		fi
		cgc -profile arbfp1 $f -entry p_main -o $fp
		cgc -profile arbvp1 $f -entry v_main -o $vp
	done
fi

export LD_LIBRARY_PATH="${PWD};${PWD}/libso"
export MANGOHUD_CONFIG="fps_limit=60;no_display=1"
LD_PRELOAD="libwal_3dxp.so" mangohud --dlsym ./main
