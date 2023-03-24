build:
	cargo build --release

install:
	install target/release/macsmc-charged /usr/local/bin/
	install macsmc-charged.service /etc/systemd/system/
