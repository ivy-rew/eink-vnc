#! /bin/sh


deps(){
	sudo apt install libevdev-dev
}

# !simulate as Elipsa 2E
# export MODEL_NUMBER=389
export PRODUCT=condor

# !libra colour
# export MODEL_NUMBER=390
export PRODUCT=monza

# !libra H20
export PRODUCT=storm

cd emulator
cargo run "$@"