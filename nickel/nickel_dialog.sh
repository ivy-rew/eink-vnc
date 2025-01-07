#!/bin/sh

defIP="192.168.1.37"
defPort="5901"
defPass="123456"

DIR=$(dirname -- "$0");

if [ -z "$(command -v qndb)" ]; then
  echo "NickelDBUS missing; unable to find qndb."
  echo "Install it from https://shermp.github.io/NickelDBus/"
  exit 1
fi

dlgStart(){
  qndb -m dlgConfirmCreate true
  qndb -m dlgConfirmSetTitle "$1"
  if [ ! -z "$2" ]; then
    qndb -m dlgConfirmSetLEPlaceholder "$2"
  fi
  qndb -m dlgConfirmSetAccept connect
  qndb -m dlgConfirmSetReject cancel
}

dlgEnd(){
  qndb -m dlgConfirmShow
  what=$(qndb -s dlgConfirmTextInput -t 60000)
  input="${what#dlgConfirmTextInput }"
  echo "${input}"
}

askDefault(){
  qndb -m dlgConfirmAcceptReject "VNC connect default" "$defIP:$defPort" "connect" "custom"
  result=$(qndb -s dlgConfirmResult)
  echo "${result#dlgConfirmResult }"
}

ask(){
  dlgStart "$1" "$2"; dlgEnd
}

askPass(){
  dlgStart "$1" "$2"
  qndb -m dlgConfirmSetLEPassword true
  dlgEnd
}

toast(){
  qndb -m mwcToast 3000 "$1"
}

ip="$defIP"
port="$defPort"
secret="$defPass"

useDef=$(askDefault)
if [ "$useDef" = 0 ]; then
  ip=$(ask "VNC connect to IP" "$defIP")
  echo "target: ${ip}"
  if [ -z "$ip" ]; then
    exit 1
  fi

  port=$(ask "VNC port on ${ip}" "$defPort")
  echo "port: ${port}"
  if [ -z "$port" ]; then
    exit 1
  fi

  secret=$(askPass "VNC password on ${ip}" "$defPass")
  echo "pass: ${secret}"
  if [ -z "$secret" ]; then
    exit 1
  fi
fi

target=${ip}:${port}
toast "connecting to ${target}"

trap ctrl_c INT
ctrl_c () {
  toast "VNC session ended"
  exit 0
}

log=$($DIR/einkvnc "${ip}" --port "${port}" --password "${secret}" --contrast 2 2>&1)
problems=$(echo "$log" | grep -E "ERROR|panic")
if [ ! -z "$problems" ]; then
  toast "VNC client for $target failed:\n $problems"
  exit 1
fi

toast "VNC session ended"
exit 0
