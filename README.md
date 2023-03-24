# macsmc_charged

A simple little daemon for managing battery charging behaviour on arm macs running (asahi) linux.

It will charge your mac up to 80% battery and then inhibit charging (i.e. run only on AC) until it goes below 70%, at which point it will allow charging up to 80% again.

If battery for some reason is at more than 80% charge, it will discharge until 80% is reached.

## Building

Make sure you have rust installed, then run `make` or the use the standard rust tooling of `cargo build`

## Installing
```
make
sudo make install
```
This will install the binary to `/usr/local/bin/macsmc-charged` and install a systemd service file to `/etc/systemd/system/macsmc-charged.service`

Start and enable the daemon with `sudo systemctl enable macsmc-charged.service --now`
