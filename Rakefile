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
    ./configure --sysconfdir=/etc --with-alsa --with-avahi --with-ssl=openssl --with-metadata --with-soxr --with-systemd
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
  SH
end

desc 'set up soundcard'
task :setup_soundcard do
  sh 'ssh', HOST, "sudo sed -i 's/defaults.ctl.card 0/defaults.ctl.card 1/' /usr/share/alsa/alsa.conf"
  sh 'ssh', HOST, "sudo sed -i 's/defaults.pcm.card 0/defaults.pcm.card 1/' /usr/share/alsa/alsa.conf"
  sh 'ssh', HOST, 'amixer', 'sset', 'Speaker', '100%'
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
