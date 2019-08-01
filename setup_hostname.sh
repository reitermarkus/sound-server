#!/bin/sh

set -e
set -o pipefail
set -o nounset

if ! dpkg -s dnsutils >/dev/null; then
  sudo apt-get update
  sudo apt-get install -y dnsutils
fi

hostname="$(dig -4 +short -x "$(hostname -I | awk '{print $1}')")"
hostname="${hostname%%.local.}"

if [ -n "${hostname}" ]; then
  echo "${hostname}" | sudo tee /etc/hostname >/dev/null
fi
