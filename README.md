## Required Packages
### Arch Linux
```
yay -S nvidia-cg-toolkit mangohud lib32-glibc lib32-gcc-libs lib32-libx11 lib32-libxcb lib32-libpulse lib32-alsa-lib lib32-libxau lib32-libxdmcp lib32-sndio lib32-libbsd lib32-libmd lib32-mangohud
```
### Debian/Ubuntu/Kali
```
sudo dpkg --add-architecture i386
sudo apt update
sudo apt install nvidia-cg-toolkit mangohud libc6:i386 libx11-dev:i386 libsndio-dev:i386 pulseaudio:i386
```
