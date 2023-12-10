use crate::protocol::meta_info_file::{Sha1HashBytes, TorrentFile};
use crate::protocol::peer_wire::PeerConnection;
use crate::protocol::tracker::{AnnounceResponse, PeerAddress, TrackerUrl};
use crate::protocol::{meta_info_file, peer_wire, tracker};
use log::debug;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::task::{JoinError, JoinSet};

/// Hard coded peer ID prefix specific to this client.
const PEER_ID_PREFIX: &str = "-RT0100-";

/// Client related errors.
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse .torrent file")]
    TorrentFileParse(#[from] meta_info_file::Error),
    #[error("http client error")]
    HttpClient(#[from] reqwest::Error),
    #[error("failed to parse http response")]
    HttpResponseParse(#[from] serde_bencode::Error),
    #[error("tracker error")]
    Tracker(#[from] tracker::Error),
    #[error("protocol error")]
    Protocol(#[from] peer_wire::Error),
    #[error("async error")]
    Async(#[from] JoinError),
    #[error("I/O error")]
    IO(#[from] io::Error),
    #[error("peer connection timeout: {0:?}")]
    PeerConnectionTimeout(Duration),
}

/// Configuration for [`BitTorrentClient`].
pub struct BitTorrentClientConfig {
    timeouts: BitTorrentClientConfigTimeouts,
}

/// Low-level networking timeout configuration.
pub struct BitTorrentClientConfigTimeouts {
    stream_connection_timeout: Duration,
    handshake_io_timeout: Duration,
}

/// BitTorrent client implementation
pub struct BitTorrentClient {
    http_client: reqwest::Client,
    peer_id: String,
    config: Arc<BitTorrentClientConfig>,
}

impl Default for BitTorrentClient {
    fn default() -> Self {
        let peer_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20 - PEER_ID_PREFIX.len())
            .map(char::from)
            .collect();
        Self {
            http_client: reqwest::Client::new(),
            peer_id: format!("{0}{1}", PEER_ID_PREFIX, peer_id),
            config: Arc::new(BitTorrentClientConfig {
                timeouts: BitTorrentClientConfigTimeouts {
                    stream_connection_timeout: Duration::from_secs(30),
                    handshake_io_timeout: Duration::from_secs(30),
                },
            }),
        }
    }
}

impl BitTorrentClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Arc<BitTorrentClientConfig>) -> Self {
        self.config = config;
        self
    }

    /// Initiates a new TCP connection to the given address and applies a connectivity timeout based on `config`.
    async fn tcp_stream_with_timeout(
        config: Arc<BitTorrentClientConfig>,
        address: String,
    ) -> Result<TcpStream, Error> {
        tokio::time::timeout(
            config.timeouts.stream_connection_timeout,
            TcpStream::connect(address),
        )
        .await
        .map_err(|_| Error::PeerConnectionTimeout(config.timeouts.stream_connection_timeout))?
        .map_err(Error::IO)
    }

    /// Initializes a connection to a torrent peer and performs handshake.
    async fn init_peer_connection(
        config: Arc<BitTorrentClientConfig>,
        peer_id: String,
        peer_address: PeerAddress,
        info_hash: Sha1HashBytes,
    ) -> Result<PeerConnection<TcpStream>, Error> {
        let mut peer_connection = PeerConnection::new(
            Self::tcp_stream_with_timeout(config.clone(), peer_address.to_string()).await?,
            config.timeouts.handshake_io_timeout,
        );
        peer_connection.handshake(peer_id, info_hash).await?;

        Ok(peer_connection)
    }

    /// Starts the download of a torrent file.
    /// Currently this method only establishes connection with all peers, performs handshake
    /// and closes connection if there was no error.
    pub async fn download(&self, torrent_file_path: &str) -> Result<(), Error> {
        // read and parse torrent file
        let torrent_file = meta_info_file::parse(torrent_file_path).await?;
        debug!("Torrent file: {:?}", torrent_file.name);

        // get peers from tracker's announce URL
        let response = self.announce(&torrent_file).await?;
        let peers = response.peers().map_err(Error::Tracker)?;

        debug!("{0} peers found!", peers.len());

        // start to connect to all peers parallel, do handshake then disconnect
        let mut handlers = JoinSet::new();
        for peer in peers {
            let peer_id = self.peer_id.clone();
            let config = self.config.clone();
            handlers.spawn(async move {
                let peer_connection =
                    Self::init_peer_connection(config, peer_id, peer, torrent_file.info_hash)
                        .await?;
                peer_connection.stream().lock().await.shutdown().await?;
                Ok::<(), Error>(())
            });
        }

        // wait for all peer connections to finish
        while let Some(res) = handlers.join_next().await {
            let result: Result<(), Error> = res.map_err(Error::Async)?;
            if result.is_err() {
                debug!("Peer connection error: {:?}", result.unwrap_err());
            }
        }

        Ok(())
    }

    /// Get all details of the torrent from the tracker parsed from .torrent file.
    async fn announce(&self, torrent: &TorrentFile) -> Result<AnnounceResponse, Error> {
        let url = TrackerUrl::new(torrent.announce.clone(), self.peer_id.clone())
            .with_compact(true)
            .with_info_hash(torrent.info_hash)
            .to_string();
        debug!("Announce URL: {:?}", url);
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(Error::HttpClient)?;
        let response_body = response.bytes().await.map_err(Error::HttpClient)?;
        let resp: AnnounceResponse = serde_bencode::from_bytes(response_body.as_ref())?;

        Ok(resp)
    }
}
