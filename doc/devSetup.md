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


## Environment

Environment variables allow configuration beyond the CLI args.

- `RUST_LOG` = [`debug`, `info`, .. ]; the log-level e.g. `export RUST_LOG=info`


## Emulator

We have an emulator for development purposes, that will simulate a real eink display.
So you do not necessary need an eink-device to run the current development state.

```shell
# start vnc: tigervnc-server is my favourite due to its speed
elipsaRes=1872x1404
vncserver -localhost no -name home -xstartup $HOME/.vnc/anonymous-vnc_xstartup -geometry 1872x1404
# connect: assume vnc is running localhost:5901 and password '123456'
./run_emulator.sh localhost --port 5901 --password 123456 --contrast 2
```

To simulate a color device https://github.com/ivy-rew/eink-vnc/pull/28, or one with another resolution, the `run_emulator.sh` must be edited. 
Export the `product` environment variable, that matches your device.
