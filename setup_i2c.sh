#!/bin/sh

set -e
set -o pipefail
set -o nounset

sudo raspi-config nonint do_i2c 0

if ! cat /etc/modules | grep -q i2c-bcm2708; then
  echo 'i2c-bcm2708' | sudo tee -a /etc/modules
fi

if ! cat /etc/modules | grep -q i2c-dev; then
  echo 'i2c-dev' | sudo tee -a /etc/modules
fi
