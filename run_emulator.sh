#! /bin/sh


deps(){
	sudo apt install libevdev-dev
}

cd emulator
cargo run "$@"
