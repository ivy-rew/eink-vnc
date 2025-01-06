# eInk VNC

A lightweight CLI (command line interface) tool to view a remote screen over VNC, designed to work on eInk screens.
For now, you can only view and use the mouse via touch-screen, so you'll have to connect a keyboard to the serving computer, 
or find some other way to interact with it.

This tool has been confirmed to work on several Kobo devices, such as the Kobo Libra 2 and Elipsa2E.
It was optimized for text based workflows (document reading and writing), doing that it achieves a framerate of 30 fps.

As VNC server we tested successfuly with TightVNC, x11vnc and TigerVNC.


## Warning

The screen can refresh up to 30 times per second, this will degrade the eInk display rapidly.
Do not use with fast changing content like videos.

Furthermore, this tool was only tested on Kobo Libra 2 and Kobo Elipsa 2E.
**It is possible that it will damage yours.**
*I cannot be held responsible, use this tool at your own risk.*

[einkvnc_demo_kobo_rotated.webm](https://user-images.githubusercontent.com/4356678/184497681-683af36b-e226-47fc-8993-34a5b356edba.webm)

## Usage

You can use this tool by connecting to the eInk device through SSH, or using menu launchers on the device itself.

To connect to a VNC server:

``` shell
./einkvnc [IP_ADDRESS] [OPTIONS]
```

For example:

``` shell
./einkvnc 192.168.2.1 --port 5902 --password abcdefg123 --contrast 2 
```

For faster framerates, use USB networking (see https://www.mobileread.com/forums/showthread.php?t=254214).

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

## Derivatives

This projects thrives due to great achievements of others... 

- Rendering for the eInk display was written by baskerville and taken from https://github.com/baskerville/plato.
- VNC protocol was written by whitequark and taken from https://github.com/whitequark/rust-vnc.
- Touch device listener was written by ewpratten and taken from https://github.com/ewpratten/kobo-rs
