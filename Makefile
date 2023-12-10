test:
	RUST_LOG=debug cargo run -p simple examples/torrents/ubuntu-desktop.torrent && RUST_LOG=debug cargo run -p simple examples/torrents/ubuntu-live-server.torrent