# ibisibi
Simple command line tool to send commands over serial port to IBIS ports, given suitable hardware.

Confirmed to work on Windows and Linux.

## Examples
To list available serial ports:
```
ibisibi list
```

To Show destination 1, then destination 0, then loop through destinations 5 to 10, then repeat:
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