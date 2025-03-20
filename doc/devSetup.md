# Development

For basic development rustup, with a stable toolchain and cross is required:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
rustup install stable
cargo install cross
```

To run the emulator the following deps must be installed:
```bash
sudo apt install -y libsdl2-dev libevdev-dev
```
