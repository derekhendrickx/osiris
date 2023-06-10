# osiris

```sh
Osiris torrent tracker

USAGE:
    osiris [OPTIONS]

OPTIONS:
    -h, --help           Print help information
    -p, --port <PORT>    Tracker port [default: 6969]
    -V, --version        Print version information
```

## Testing

```sh
echo -e '\x0''\x0''\x4''\x17''\x27''\x10''\x19''\x80''\x0''\x0''\x0''\x0''\x0''\x0''\x0''\x2A' | nc -u 127.0.0.1 6969
```

## Resources

* https://www.bittorrent.org/beps/bep_0015.html
* https://www.rasterbar.com/products/libtorrent/udp_tracker_protocol.html
