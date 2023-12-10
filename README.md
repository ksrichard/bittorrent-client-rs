# BitTorrent Handshake

This Rust crate implements a basic BitTorrent Client to connect to peers (via `Peer Wire protocol`) and perform valid handshake with them.

## Features
 - Read and parse .torrent files
 - Communication with torrent tracker to get all bittorrent peers (only http now)
 - Connect to all peers parallel through TCP connection and perform handshake as initial steps of download process

## Usage
This library is very simple to use. There is a `BitTorrentClient` struct which has a `download` method (accepts a `.torrent` file as an input)
to parse the passed `.torrent` file, then creates peer-to-peer connections to all torrent peers and performs handshake with them (validates the response handshake as well),
then closes connections.

### Example:
```rust
use client::{BitTorrentClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let client = BitTorrentClient::new();
    client.download("example.torrent").await
}
```

## Testing

### Prerequisites
- Install [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- **(Optional)** [Install Docker](https://docs.docker.com/engine/install/)

- ### Steps
  - Open a new terminal
  - Go to the root of the project directory
  - Run `simple` example from [examples folder](./examples): 
    - `$ RUST_LOG=debug cargo run -p simple examples/torrents/ubuntu-desktop.torrent` or
    - `$ make test` - to run both handshakes for all torrents
  - In the logs you will see that:
    - what is the file parsed from the torrent file to be downloaded
    - what is the tracker announce URL
    - all handshake related debug messages
  - Example output:
```shell
[2023-12-10T15:26:29Z DEBUG client::client] Torrent file: "debian-12.2.0-amd64-DVD-1.iso"
[2023-12-10T15:26:29Z DEBUG client::client] Announce URL: "http://bttracker.debian.org:6969/announce?uploaded=0&left=0&downloaded=0&compact=1&info_hash=%26%7Dc%FF%D3%17p%E4g%F8%D9%85%A8f3%F0U%02%C1%0D&peer_id=-RT0100-kOIX1vXBDujy&port=6881"
[2023-12-10T15:26:29Z DEBUG reqwest::connect] starting new connection: http://bttracker.debian.org:6969/
[2023-12-10T15:26:30Z DEBUG client::client] 41 peers found!
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [5.146.170.236:63456] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [85.195.233.40:43690] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [185.21.216.191:58552] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [45.91.208.190:55288] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [88.150.13.146:51443] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [194.127.178.48:50068] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [134.249.136.112:29905] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.233.218.31:52539] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.84.78.247:56056] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [194.213.3.203:51413] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::client] Peer connection error: IO(Os { code: 61, kind: ConnectionRefused, message: "Connection refused" })
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.84.78.247:56057] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [85.195.233.40:43690] handshake response received: HandshakeMessage { peer_id: "-qB4600-x_SyUYaWR*3g", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [85.195.233.40:43690] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [185.21.216.191:58552] handshake response received: HandshakeMessage { peer_id: "-DE13F0-F8)mZ6e~5Dgu", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [185.21.216.191:58552] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::client] Peer connection error: IO(Os { code: 61, kind: ConnectionRefused, message: "Connection refused" })
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [45.91.208.190:55288] handshake response received: HandshakeMessage { peer_id: "-qB4390-Cl!5D3nBXbSD", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [45.91.208.190:55288] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [5.146.170.236:63456] handshake response received: HandshakeMessage { peer_id: "-qB4620-g1C6UWjuQ8Mf", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [5.146.170.236:63456] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [194.127.178.48:50068] handshake response received: HandshakeMessage { peer_id: "-qB4620-9K_O3ddtJwp3", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [194.127.178.48:50068] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [23.137.253.6:37159] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.233.218.31:52539] handshake response received: HandshakeMessage { peer_id: "-DE13F0--GyISj)-_nly", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.233.218.31:52539] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [76.68.130.197:56090] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::client] Peer connection error: IO(Os { code: 61, kind: ConnectionRefused, message: "Connection refused" })
[2023-12-10T15:26:30Z DEBUG client::client] Peer connection error: Protocol(InvalidHandshakeMessageBytesLength)
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [146.70.198.51:35198] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [192.145.130.102:51413] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [108.201.79.41:58763] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [134.249.136.112:29905] handshake response received: HandshakeMessage { peer_id: "-TR3000-qhtlb83r4dl0", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [134.249.136.112:29905] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [23.137.253.6:37159] handshake response received: HandshakeMessage { peer_id: "-qB4550-d5H7si4xF-o7", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [23.137.253.6:37159] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::client] Peer connection error: Protocol(InvalidHandshakeMessageBytesLength)
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [118.36.225.146:5555] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.84.78.247:56056] handshake response received: HandshakeMessage { peer_id: "-qB4600-aWxJ-HCSEU(Y", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [83.84.78.247:56056] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [180.150.41.117:9843] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [59.13.25.153:6881] start handshake with peer...
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [194.213.3.203:51413] handshake response received: HandshakeMessage { peer_id: "-TR410Z-b1iv4axex7yk", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [194.213.3.203:51413] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [88.150.13.146:51443] handshake response received: HandshakeMessage { peer_id: "-TR3000-hzhgc2ipi53c", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [88.150.13.146:51443] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::client] Peer connection error: Protocol(InvalidHandshakeMessageBytesLength)
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [192.145.130.102:51413] handshake response received: HandshakeMessage { peer_id: "-TR4040-xfegwbmksxp3", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [192.145.130.102:51413] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [118.36.225.146:5555] handshake response received: HandshakeMessage { peer_id: "-qB4610-QnyVA~-w0k7r", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [118.36.225.146:5555] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [180.150.41.117:9843] handshake response received: HandshakeMessage { peer_id: "-DE211s-6f(*Jh8-Z)7Y", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [180.150.41.117:9843] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [59.13.25.153:6881] handshake response received: HandshakeMessage { peer_id: "-UW1409-RhyNMT4frzPb", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [59.13.25.153:6881] handshake is valid
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [146.70.198.51:35198] handshake response received: HandshakeMessage { peer_id: "-TR4040-qvi50b204d0y", info_hash: [38, 125, 99, 255, 211, 23, 112, 228, 103, 248, 217, 133, 168, 102, 51, 240, 85, 2, 193, 13], protocol_id: "BitTorrent protocol" }
[2023-12-10T15:26:30Z DEBUG client::protocol::peer_wire] [146.70.198.51:35198] handshake is valid
```



