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
ibisibi cycle 1 0 --from 5 --to 10 --serial <port from ibisibi list>
```