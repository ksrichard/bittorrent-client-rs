use client::{BitTorrentClient, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let args = std::env::args();
    if args.len() <= 1 {
        panic!("Please provide a torrent file as first argument!");
    }
    let torrent_file = args.last().unwrap();
    let client = BitTorrentClient::new();
    client.download(torrent_file.as_str()).await
}
