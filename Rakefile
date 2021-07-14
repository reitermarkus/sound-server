require 'securerandom'
require 'shellwords'

RPI = ENV['RPI'] || 'sound-server.local'
HOST = "pi@#{RPI}"

def ssh(*args)
  sh 'ssh', HOST, *args
end

desc 'set time zone on Raspberry Pi'
task :setup_timezone do
  ssh 'sudo', 'timedatectl', 'set-timezone', 'Europe/Vienna'
end

desc 'set hostname on Raspberry Pi'
task :setup_hostname do
  ssh <<~SH
    if ! dpkg -s dnsutils >/dev/null; then
      sudo apt-get update
      sudo apt-get install -y dnsutils
    fi

    hostname="$(dig -4 +short -x "$(hostname -I | awk '{print $1}')")"
    hostname="${hostname%%.local.}"

    if [ -n "${hostname}" ]; then
      echo "${hostname}" | sudo tee /etc/hostname >/dev/null
    fi
  SH
end

desc 'set up I2C on Raspberry Pi'
task :setup_i2c do
  ssh 'sudo', 'raspi-config', 'nonint', 'do_i2c', '0'

  IO.pipe do |r, w|
    w.puts <<~CFG
      SUBSYSTEM=="i2c-dev", ATTR{name}=="bcm2835 I2C adapter", SYMLINK+="i2c", TAG+="systemd"
    CFG
    w.close

    ssh 'sudo', 'tee', '/lib/udev/rules.d/99-i2c.rules', in: r
  end
end

desc 'set up watchdog on Raspberry Pi'
task :setup_watchdog do
  ssh <<~SH
    if ! dpkg -s watchdog >/dev/null; then
      sudo apt-get update
      sudo apt-get install -y watchdog
    fi
  SH

  IO.pipe do |r, w|
    w.puts 'bcm2835_wdt'
    w.close

    ssh 'sudo', 'tee', '/etc/modules-load.d/bcm2835_wdt.conf', in: r
  end

  gateway_ip = %x(#{['ssh', HOST, 'ip', 'route'].shelljoin})[/via (\d+.\d+.\d+.\d+) /, 1]
  raise if gateway_ip.empty?

  IO.pipe do |r, w|
    w.puts <<~CFG
      watchdog-device	= /dev/watchdog
      ping = #{gateway_ip}
    CFG
    w.close

    ssh 'sudo', 'tee', '/etc/watchdog.conf', in: r
    ssh 'sudo', 'systemctl', 'enable', 'watchdog'
  end
end

desc 'set up shairport'
task :setup_shairport do
  ssh <<~'SH'
    set -euo pipefail

    # Turn off Wi-Fi power management.
    sudo iwconfig wlan0 power off

    # Install dependencies.
    sudo apt-get update
    sudo apt-get install -y build-essential \
                            git \
                            xmltoman \
                            autoconf \
                            automake \
                            avahi-daemon \
                            libtool \
                            libpopt-dev \
                            libconfig-dev \
                            libasound2-dev \
                            libavahi-client-dev \
                            libmosquitto-dev \
                            libssl-dev \
                            libsoxr-dev

    # Uninstall previous version.
    sudo rm -f /usr/local/bin/shairport-sync
    sudo rm -f /etc/systemd/system/shairport-sync*.service
    sudo rm -f /etc/init.d/shairport-sync*

    pushd /tmp
    rm -rf shairport-sync-build
    git clone https://github.com/mikebrady/shairport-sync.git shairport-sync-build
    pushd shairport-sync-build
    autoreconf -fi
    ./configure --sysconfdir=/etc --with-alsa --with-avahi --with-ssl=openssl --with-metadata --with-soxr --with-systemd --with-mqtt-client
    make
    sudo make install
    popd
    rm -r shairport-sync-build
    popd

    sudo cp -f /lib/systemd/system/shairport-sync.service /lib/systemd/system/shairport-sync@.service
    sudo sed -i -E \
      's|^(Description=.*Receiver)$|\1 (%I)|;s|^(ExecStart=/usr/local/bin/shairport-sync)$|\1 --use-stderr --verbose --configfile=/etc/shairport-sync-%I.conf\nSyslogIdentifier=shairport-sync-%I|' \
      /lib/systemd/system/shairport-sync@.service

    if ! [ -f /etc/shairport-sync.conf.sample ]; then
      sudo cp /etc/shairport-sync.conf /etc/shairport-sync.conf.sample
    fi

    sudo adduser shairport-sync gpio
  SH

  [
    ["Garage", "garage", "USB_Card_Garage", 5],
    ["Garten", "garden", "USB_Card_Garden", 6],
  ].each do |name, id, sound_card, gpio|
    [
      ["before", 0],
      ["after", 1],
    ].each do |trigger, value|
      IO.pipe do |r, w|
        w.puts <<~SH
          #!/usr/bin/env bash

          set -euo pipefail

          [[ -e /sys/class/gpio/gpio#{gpio} ]] || echo #{gpio} > /sys/class/gpio/export
          echo out > /sys/class/gpio/gpio#{gpio}/direction
          echo #{value} > /sys/class/gpio/gpio#{gpio}/value
        SH
        w.close

        ssh 'sudo', 'tee', "/etc/shairport-sync-#{id}-#{trigger}.sh", in: r
      end
      ssh 'sudo', 'chmod', '+x', "/etc/shairport-sync-#{id}-#{trigger}.sh"
    end

    IO.pipe do |r, w|
      w.puts <<~CFG
        general = {
          name = "#{name}";

          port = #{5000 + gpio};

          playback_mode = "mono";
        };

        sessioncontrol = {
          run_this_before_play_begins = "/etc/shairport-sync-#{id}-before.sh";
          run_this_after_play_ends = "/etc/shairport-sync-#{id}-after.sh";
          wait_for_completion = "yes";
        };

        alsa = {
          output_device = "hw:#{sound_card}";
        };
      CFG
      w.close

      ssh 'sudo', 'tee', "/etc/shairport-sync-#{id}.conf", in: r
    end

    ssh <<~SH
      sudo systemctl enable shairport-sync@#{id}
      sudo systemctl start shairport-sync@#{id}
    SH
  end
end

desc 'set up sound card names'
task :setup_soundcards do
  r, w = IO.pipe

  w.puts <<~CFG
    ACTION=="add", SUBSYSTEM=="sound", ATTRS{idVendor}=="0d8c", ATTRS{idProduct}=="0014", KERNELS=="1-1.2", ATTR{id}="USB_Card_Garage"
    ACTION=="add", SUBSYSTEM=="sound", ATTRS{idVendor}=="0d8c", ATTRS{idProduct}=="0014", KERNELS=="1-1.4", ATTR{id}="USB_Card_Garden"
  CFG
  w.close

  ssh 'sudo', 'tee', '/lib/udev/rules.d/85-usb-soundcard.rules', in: r

  ssh 'amixer', '-D', 'hw:USB_Card_Garage', 'sset', 'Speaker', '100%'
  ssh 'amixer', '-D', 'hw:USB_Card_Garden', 'sset', 'Speaker', '100%'
end
