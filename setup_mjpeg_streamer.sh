#!/bin/sh

set -e
set -o pipefail
set -o nounset
set -x

sudo raspi-config nonint do_camera 0

if ! cat /etc/modules | grep -q bcm2835-v4l2; then
  echo 'bcm2835-v4l2' | sudo tee -a /etc/modules
fi

if ! [ -x /usr/local/bin/mjpg_streamer ]; then
  sudo apt-get update

  sudo apt-get install -y cmake libjpeg8-dev

  rm -rf /tmp/mjpg-streamer
  git clone https://github.com/jacksonliam/mjpg-streamer /tmp/mjpg-streamer

  pushd /tmp/mjpg-streamer/mjpg-streamer-experimental

  make
  sudo make install

  popd
fi

cat <<CONFIG | sudo tee /etc/systemd/system/mjpg-streamer.service
[Unit]
Description=mjpg-streamer
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/mjpg_streamer -i "input_raspicam.so --width 1920 --height 1440 -quality 100 --framerate 30 -sh 100 -co 100 -br 75 -ex auto -awb auto -drc high" -o "output_http.so -p 8080 -w /usr/local/share/mjpg-streamer/www"
Restart=always

[Install]
WantedBy=multi-user.target
CONFIG

sudo systemctl enable mjpg-streamer
sudo systemctl restart mjpg-streamer
