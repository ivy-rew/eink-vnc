# Building

I've had success with the following commands

```
docker pull ewpratten/kobo-cross-armhf:latest
cargo install cross
cross build --target arm-unknown-linux-musleabihf
```

Steps from scratch....

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add arm-unknown-linux-musleabihf
rustup update
cd client
cargo build --target=arm-unknown-linux-musleabihf
```

Compilation currently fails on a linker error - /usr/bin/ld: /home/andrew/.rustup/toolchains/stable-aarch64-unknown-linux-gnu/lib/rustlib/arm-unknown-linux-musleabihf/lib/self-contained/crt1.o: error adding symbols: file in wrong format
 
/usr/bin/ld is obviously the wrong linker.  But, also, a copy of the 
arm-unknown-linux-musleabihf SDL2 library is needed...  Possibly in 
.rustup/toolchains/stable-aarch64-unknown-linux-gnu/lib/rustlib/arm-unknown-linux-musleabihf/lib/self-contained, or maybe
just somewhere that the cross compiler knows about it.

Trying https://wiki.osdev.org/Building_GCC

# eInk VNC

A lightweight CLI (command line interface) tool to view a remote screen over VNC, designed to work on eInk screens.
For now, you can only view, so you'll have to connect a keyboard to the serving computer, or find some other way to interact with it.

This tool has been confirmed to work on several Kobo devices, such as the Kobo Libra 2 and Elipsa2E.
It was optimized for text based workflows (document reading and writing), doing that it achieves a framerate of 30 fps.

**It has only been confirmed to work with TightVNC as the server.  Sort of.**
Due to the unusual pixel format.

The source in the repository has "worked" on other VNC servers - tigervnc
in particular.  I coded a dirty RGBA to greyscale converter

```
                    let scale_down =
                        pixels
                            .iter()
                            .step_by(4)
                            .map(|&c| post_proc_bin.data[c as usize])
                            .collect();
```

i.e., the output color is basically the red channel.

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
./einkvnc [IP_ADDRESS] [PORT] [OPTIONS]
```

For example:

``` shell
./einkvnc 192.168.2.1 5902 --password abcdefg123 --contrast 2 
```

For faster framerates, use USB networking (see https://www.mobileread.com/forums/showthread.php?t=254214).

## Derivatives

The code responsible for rendering to the eInk display is written by baskerville and taken from https://github.com/baskerville/plato.
The code responsible for communicating using the VNC protocol is written by whitequark and taken from https://github.com/whitequark/rust-vnc.
