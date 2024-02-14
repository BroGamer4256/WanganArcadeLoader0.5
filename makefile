all:
	@PKG_CONFIG_SYSROOT_DIR=/usr/lib32/pkgconfig/ cargo b --release --target i686-unknown-linux-gnu

dist-no-7z: all
	@mkdir -p out/
	@cp target/i686-unknown-linux-gnu/release/libwal_3dxp.so out/
	@cp -r dist/* out/

dist: dist-no-7z
	@cd out && 7z a -t7z ../dist.7z .
	@rm -rf out
