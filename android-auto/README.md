# OpenAuto

An Android Auto head-unit emulator. This repository builds the full stack
(AASDK + OpenAuto) entirely into a local `./output` directory — nothing is
installed to system paths.

## Dependencies (Arch Linux)

Install everything for a full build-and-run machine:

```bash
sudo pacman -S --needed base-devel cmake protobuf openssl libusb rtaudio \
    boost qt5-base qt5-multimedia qt5-connectivity
```

Optional: `ninja` (for faster builds), `pkg-config`.

### Runtime dependencies

Required to run `autoapp` / `btservice` (loaded via dynamic linking):

| Package | Provides |
|---|---|
| `base-devel` | gcc, make, etc. required for any build |
| `cmake` | build system |
| `ninja` (opt) | faster builds |
| `pkg-config` (opt) | dependency detection |
| `protobuf` | libprotobuf used by the AASDK messages |
| `openssl` | libssl / libcrypto — TLS for the Android Auto connection |
| `libusb` | USB host-mode detection |
| `qt5-base` | Qt5Widgets/Gui/Core used by the UI |
| `qt5-multimedia` | Qt5Multimedia / Qt5MultimediaWidgets |
| `qt5-connectivity` | Qt5Bluetooth for A2DP/Bluetooth audio |
| `rtaudio` | Audio output backend |