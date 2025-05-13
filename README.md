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

## User Guide 📸️🤩️💫️

In the [Kobo Setup](doc/koboSetup.md) guide the basic installation steps
are described to run the tool on your Kobo eReader 🚀️. It covers topis such as the best VNC server to use, and recommendations on the network connection.

If you feel like contributing to the development, please refer to the [Dev Setup](doc/devSetup.md) guide.

## Derivatives

This projects thrives due to great achievements of others... 

- Rendering for the eInk display was written by baskerville and taken from https://github.com/baskerville/plato.
- VNC protocol was written by whitequark and taken from https://github.com/whitequark/rust-vnc.
- Touch device listener was written by ewpratten and taken from https://github.com/ewpratten/kobo-rs
