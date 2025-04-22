#!/bin/sh

DIR=$(dirname -- "$0");
. "${DIR}/default_client.sh"

if [ -z "$ip" ]; then
  ip=192.168.1.48
fi

if [ -z "$port" ]; then
  port=5901
fi

alias vnc="$DIR/einkvnc $ip --port $port --password 123456 --touch $KOBO_TS_INPUT --rotate $defRotation"
