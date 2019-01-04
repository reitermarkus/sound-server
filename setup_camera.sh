#!/bin/sh

set -e
set -o pipefail
set -o nounset

sudo apt-get update

sudo apt-get install -y motion

if ! [ -f /etc/motion/motion.conf.sample ]; then
  sudo cp /etc/motion/motion.conf /etc/motion/motion.conf.sample
fi

if ! cat /etc/modules | grep -q bcm2835-v4l2; then
  echo 'bcm2835-v4l2' | sudo tee -a /etc/modules
fi

sudo sed -i -e 's/daemon off/daemon on/' /etc/motion/motion.conf
sudo sed -i -e 's/stream_localhost on/stream_localhost off/' /etc/motion/motion.conf
sudo sed -i -e 's/width 320/width 640/' /etc/motion/motion.conf
sudo sed -i -e 's/height 240/height 480/' /etc/motion/motion.conf
sudo sed -i -e 's/framerate 2/framerate 60/' /etc/motion/motion.conf
sudo sed -i -e 's/stream_maxrate 1/stream_maxrate 60/' /etc/motion/motion.conf

sudo sed -i -e 's/start_motion_daemon=no/start_motion_daemon=yes/' /etc/default/motion

sudo service motion start
