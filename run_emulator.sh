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

# avoid linker errors due to different target envs present for the 'client'
unset CARGO_BUILD_TARGET

cd emulator
cargo run "$@"