#!/bin/sh

defIP="10.42.0.1"
defPort="5901"
defPass="123456"

defContrast=3.0
defRotation=4

# hardcode your product; enhance detection of correct device
#export PRODUCT=condor

# wire elipsa 2E
export KOBO_TS_INPUT=/dev/input/event2
