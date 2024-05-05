all:
	@PKG_CONFIG_SYSROOT_DIR=/usr/lib32/pkgconfig/ cargo b --release --target i586-unknown-linux-gnu

check:
	@PKG_CONFIG_SYSROOT_DIR=/usr/lib32/pkgconfig/ cargo clippy --release --target i586-unknown-linux-gnu -- -A clippy::not_unsafe_ptr_arg_deref -A clippy::missing_safety_doc -A clippy::implicit_saturating_sub -A clippy::missing_transmute_annotations --no-deps

dist-no-7z: all
	@mkdir -p out/
	@cp target/i586-unknown-linux-gnu/release/libwal_3dxp.so out/
	@cp -r dist/* out/

dist: dist-no-7z
	@cd out && 7z a -t7z ../dist.7z .
	@rm -rf out
