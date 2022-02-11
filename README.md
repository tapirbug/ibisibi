# ibisibi
Simple command line tool to send commands over serial port to IBIS ports, given suitable hardware.

Confirmed to work on Windows and Linux.

## Install
You can download a release tarball from Github for your platform.
On linux platforms you can use the install script to run [`examples/robo.yaml`](examples/robo.yaml) as a
systemd service, e.g. to set up on Raspberry Pi log in via ssh and run:
```
wget -c https://github.com/tapirbug/ibisibi/releases/download/0.3.0/ibisibi-0.3.0-arm-unknown-linux-gnueabihf.tar.gz -O - \
| tar -xz && \
cd ibisibi-0.3.0-arm-unknown-linux-gnueabihf && \
./install.sh
```
You may be required to enter your password when the install script installs the `ibisibi.service` unit file.

## Examples
To list available serial ports:
```
$ ibisibi list
/dev/ttyUSB0
```

To scan for devices and print their statuses and addresses on a given serial port:
```
$ ibisibi scan <port from ibisibi list>
1: Ok (3)
```

To flash a database to a device with a given address:
```
# Warning: This not only overwrites the currently flashed data,
# this is also very very experimental -  Use at your own risk!
$ ibisibi flash some_db.hex --address <Address from scan, e.g. "1"> --serial <port from ibisibi list>
[... Debug output will be written ...]
```

To show destination 1, then destination 0, then loop through destinations 5 to 10, then repeat, on all listening devices:
```
ibisibi cycle 1 0 5-10 --serial <port from ibisibi list>
```

Destinations can be associated with a timestamp. If the timestamp is in the past or more than a
specified amount of hours into the future, then the associated destination will not yet or no longer
be shown, e.g.:
```
ibisibi cycle \
# Show entries up to 12 hours in advance
--lookahead 12
# Show this one regardless of time
0 \
# Show 2,4,5 only if in the relevant time range
2@2021-09-09T20:00:00/2021-09-09T21:00:00 \
4@2021-09-10T17:00:00/2021-09-10T21:00:00 \
5@2021-09-10T21:00:00/2021-09-10T23:00:00 \
--serial <port from ibisibi list>
```

Having a lot of destinations planned can become a bit complicated, so consider
writing a config file instead:
```
cycle:
  serial: "/dev/ttyUSB0"
  # Show a new destination every 9 seconds
  interval_secs: 9
  # Show events that are running or start in the next 12 minutes
  lookahead: 12
  plan:
    # ROBOEXOTICA (shown every day)
    - destinations:
        - 0
    # 18:00 - 24:00 Exhibition
    - destinations:
        - 6
      slots:
        - 2021-09-09T18:00:00/2021-09-10T00:00:00
```
and run it with the `run` command:
```
ibisibi run /path/to/your/config.yaml
```