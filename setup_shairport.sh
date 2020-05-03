#!/bin/sh

set -e
set -o pipefail
set -o nounset

# Turn off Wi-Fi power management.
sudo iwconfig wlan0 power off

sudo apt-get update

sudo apt-get install -y build-essential \
                        git \
                        xmltoman \
                        autoconf \
                        automake \
                        libtool \
                        libpopt-dev \
                        libconfig-dev \
                        libasound2-dev \
                        avahi-daemon \
                        libavahi-client-dev \
                        libssl-dev \
                        libsoxr-dev

# Uninstall previous version.
sudo rm -f /usr/local/bin/shairport-sync
sudo rm -f /etc/systemd/system/shairport-sync.service
sudo rm -f /etc/init.d/shairport-sync

pushd /tmp
git clone https://github.com/mikebrady/shairport-sync.git
pushd shairport-sync
autoreconf -fi
./configure --sysconfdir=/etc --with-alsa --with-soxr --with-avahi --with-ssl=openssl --with-systemd
make
sudo make install
popd
rm -r shairport-sync
popd

if ! [ -f /etc/shairport-sync.conf.sample ]; then
  sudo cp /etc/shairport-sync.conf /etc/shairport-sync.conf.sample
fi

sudo adduser shairport-sync gpio

cat <<CONFIG | sudo tee /etc/shairport-sync.conf
general =
{
  name = "Garage";
};

sessioncontrol =
{
  run_this_before_play_begins = "/usr/local/bin/garage-speakers on";
  run_this_after_play_ends = "/usr/local/bin/garage-speakers off";
  wait_for_completion = "yes";
};
CONFIG

sudo systemctl enable shairport-sync
sudo systemctl start shairport-sync
