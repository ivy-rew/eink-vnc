#! /bin/sh


deps(){
	sudo apt install libevdev-dev
}

# simulate as Elipsa 2E
export PRODUCT=condor
export MODEL_NUMBER=389

cd emulator
cargo run "$@"
