#!/bin/sh

set -e
set -o pipefail
set -o nounset

if ! dpkg -s watchdog >/dev/null; then
  sudo apt-get update
  sudo apt-get install -y watchdog
fi

if ! cat /etc/modules | grep -q bcm2835_wdt; then
  echo 'bcm2835_wdt' | sudo tee -a /etc/modules
fi

if ! [ -f /etc/watchdog.conf.sample ]; then
  sudo cp /etc/watchdog.conf /etc/watchdog.conf.sample
fi

gateway_ip="$(ip route | awk '/^default via ([^\s]+) / { print $3 }')"

[ -n "${gateway_ip}" ]

cat <<CONFIG | sudo tee /etc/watchdog.conf
watchdog-device	= /dev/watchdog
watchdog-timeout = 10
interval = 2
max-load-1 = 24
ping = ${gateway_ip}
CONFIG

sudo systemctl enable watchdog.service
