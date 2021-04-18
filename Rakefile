require 'securerandom'
require 'shellwords'

TARGET = ENV['TARGET'] || 'armv7-unknown-linux-gnueabihf'

RPI = ENV['RPI'] || 'garage.local'
HOST = "pi@#{RPI}"

def ssh(*args)
  sh 'ssh', HOST, *args
end

desc 'compile binary'
task :build do
  sh 'cross', 'build', '--release', '--target', TARGET
end

desc 'set time zone on Raspberry Pi'
task :setup_timezone do
  sh 'ssh', HOST, 'sudo', 'timedatectl', 'set-timezone', 'Europe/Vienna'
end

desc 'set hostname on Raspberry Pi'
task :setup_hostname do
  sh 'ssh', HOST, <<~SH
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
  sh 'ssh', HOST, 'sudo', 'raspi-config', 'nonint', 'do_i2c', '0'

  r, w = IO.pipe

  w.puts <<~CFG
    SUBSYSTEM=="i2c-dev", ATTR{name}=="bcm2835 I2C adapter", SYMLINK+="i2c", TAG+="systemd"
  CFG
  w.close

  sh 'ssh', HOST, 'sudo', 'tee', '/lib/udev/rules.d/99-i2c.rules', in: r
end

desc 'set up watchdog on Raspberry Pi'
task :setup_watchdog do
  sh 'ssh', HOST, <<~SH
    if ! dpkg -s watchdog >/dev/null; then
      sudo apt-get update
      sudo apt-get install -y watchdog
    fi
  SH

  r, w = IO.pipe

  w.puts 'bcm2835_wdt'
  w.close

  sh 'ssh', HOST, 'sudo', 'tee', '/etc/modules-load.d/bcm2835_wdt.conf', in: r

  gateway_ip = %x(#{['ssh', HOST, 'ip', 'route'].shelljoin})[/via (\d+.\d+.\d+.\d+) /, 1]

  raise if gateway_ip.empty?

  r, w = IO.pipe

  w.puts <<~CFG
    watchdog-device	= /dev/watchdog
    ping = #{gateway_ip}
  CFG
  w.close

  sh 'ssh', HOST, 'sudo', 'tee', '/etc/watchdog.conf', in: r
  sh 'ssh', HOST, 'sudo', 'systemctl', 'enable', 'watchdog'
end

desc 'set up mJPEG Streamer'
task :setup_mjpeg_streamer do
  sh 'ssh', HOST, <<~'SH'
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
  SH
end

desc 'set up shairport'
task :setup_shairport do
  sh 'ssh', HOST, <<~'SH'
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
    sudo rm -f /etc/systemd/system/shairport-sync*.service
    sudo rm -f /etc/init.d/shairport-sync*

    pushd /tmp
    git clone https://github.com/mikebrady/shairport-sync.git
    pushd shairport-sync
    autoreconf -fi
    ./configure --sysconfdir=/etc --with-alsa --with-avahi --with-ssl=openssl --with-metadata --with-soxr --with-systemd
    make
    sudo make install
    popd
    rm -r shairport-sync
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

        sh 'ssh', HOST, 'sudo', 'tee', "/etc/shairport-sync-#{id}-#{trigger}.sh", in: r
      end
      sh 'ssh', HOST, 'sudo', 'chmod', '+x', "/etc/shairport-sync-#{id}-#{trigger}.sh"
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

      sh 'ssh', HOST, 'sudo', 'tee', "/etc/shairport-sync-#{id}.conf", in: r
    end

    sh 'ssh', HOST, <<~SH
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

  sh 'ssh', HOST, 'sudo', 'tee', '/lib/udev/rules.d/85-usb-soundcard.rules', in: r

  sh 'ssh', HOST, 'amixer', '-D', 'hw:USB_Card_Garage', 'sset', 'Speaker', '100%'
  sh 'ssh', HOST, 'amixer', '-D', 'hw:USB_Card_Garden', 'sset', 'Speaker', '100%'
end

task :setup => [:setup_timezone, :setup_hostname, :setup_i2c, :setup_watchdog, :setup_mjpeg_streamer, :setup_shairport, :setup_soundcard]

desc 'deploy binary and service configuration to Raspberry Pi'
task :deploy => :build  do
  sh 'rsync', '-z', '--rsync-path', 'sudo rsync', "target/#{TARGET}/release/garage", "#{HOST}:/usr/local/bin/garage"
  sh 'rsync', '-z', '--rsync-path', 'sudo rsync', "target/#{TARGET}/release/garage-speakers", "#{HOST}:/usr/local/bin/garage-speakers"

  r, w = IO.pipe

  w.puts <<~CFG
    [Unit]
    StartLimitAction=reboot
    StartLimitIntervalSec=60
    StartLimitBurst=10
    Description=garage

    [Service]
    Type=simple
    Environment=I2C_DEVICE=/dev/i2c
    Environment=RUST_LOG=info
    ExecStart=/usr/local/bin/garage
    Restart=always
    RestartSec=1

    [Install]
    WantedBy=multi-user.target
  CFG
  w.close

  sh 'ssh', HOST, 'sudo', 'tee', '/etc/systemd/system/garage.service', in: r
  sh 'ssh', HOST, 'sudo', 'systemctl', 'enable', 'garage'
  sh 'ssh', HOST, 'sudo', 'systemctl', 'restart', 'garage'
end

desc 'show service log'
task :log do
  sh 'ssh', HOST, '-t', 'journalctl', '-f', '-u', 'garage'
end
