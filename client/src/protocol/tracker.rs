use crate::protocol::meta_info_file::Sha1HashBytes;
use byteorder::{BigEndian, ReadBytesExt};
use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::io::Cursor;
use std::net::{AddrParseError, IpAddr};
use thiserror::Error;
use urlencoding::encode_binary;

/// Compact peers list's (binary) peer address IP bytes length (https://wiki.theory.org/BitTorrentSpecification#Tracker_Response).
const COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH: usize = 4;

/// Compact peers list's (binary) peer address port bytes length (big endian, unsigned 16-bit) (https://wiki.theory.org/BitTorrentSpecification#Tracker_Response).
const COMPACT_PEER_ADDRESS_PORT_BYTES_LENGTH: usize = 2;

/// Compact peers list's (binary) peer address bytes length (https://wiki.theory.org/BitTorrentSpecification#Tracker_Response).
const COMPACT_PEER_ADDRESS_BYTES_LENGTH: usize =
    COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH + COMPACT_PEER_ADDRESS_PORT_BYTES_LENGTH;

/// Errors from tracker related operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse IP address")]
    IPAddressParseFailed(#[from] AddrParseError),
    #[error("invalid length of bytes of peer address: {0}")]
    PeerAddressInvalidLength(usize),
    #[error("I/O error")]
    IO(#[from] io::Error),
}

/// Tracker URL.
pub struct TrackerUrl {
    announce_url: String,
    info_hash: Sha1HashBytes,
    peer_id: String,
    port: u16,
    bytes_uploaded: usize,
    bytes_downloaded: usize,
    left_bytes: usize,
    compact: bool,
    tracker_id: Option<usize>,
}

impl TrackerUrl {
    pub fn new(announce_url: String, peer_id: String) -> Self {
        Self {
            announce_url,
            info_hash: Default::default(),
            peer_id,
            port: 6881,
            bytes_uploaded: 0,
            bytes_downloaded: 0,
            left_bytes: 0,
            compact: false,
            tracker_id: None,
        }
    }

    pub fn with_peer_id(mut self, peer_id: String) -> Self {
        self.peer_id = peer_id.to_string();
        self
    }

    pub fn with_info_hash(mut self, info_hash: Sha1HashBytes) -> Self {
        self.info_hash = info_hash;
        self
    }
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_bytes_uploaded(mut self, bytes_uploaded: usize) -> Self {
        self.bytes_uploaded = bytes_uploaded;
        self
    }

    pub fn with_bytes_downloaded(mut self, bytes_downloaded: usize) -> Self {
        self.bytes_downloaded = bytes_downloaded;
        self
    }

    pub fn with_left_bytes(mut self, left_bytes: usize) -> Self {
        self.left_bytes = left_bytes;
        self
    }

    pub fn with_compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn with_tracker_id(mut self, tracker_id: Option<usize>) -> Self {
        self.tracker_id = tracker_id;
        self
    }
}

/// Serialize [`TrackerUrl`] into a full URL.
impl ToString for TrackerUrl {
    fn to_string(&self) -> String {
        let compact = self.compact as i32;
        let compact = compact.to_string();
        let mut query_params = HashMap::from([
            (
                "info_hash",
                encode_binary(self.info_hash.as_slice()).to_string(),
            ),
            ("peer_id", self.peer_id.clone()),
            ("port", self.port.to_string()),
            ("uploaded", self.bytes_uploaded.to_string()),
            ("downloaded", self.bytes_downloaded.to_string()),
            ("left", self.left_bytes.to_string()),
            ("compact", compact),
        ]);
        if let Some(tracker_id) = self.tracker_id {
            query_params.insert("trackerid", tracker_id.to_string());
        }
        let query_params: Vec<String> = query_params
            .iter()
            .map(|(k, v)| format!("{0}={1}", k, v))
            .collect();
        let query_params = query_params.join("&");
        format!("{0}?{1}", self.announce_url, query_params)
    }
}

/// Announce URL response struct.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
pub struct AnnounceResponse {
    #[serde(rename = "failure reason")]
    failure_reason: Option<String>,
    #[serde(rename = "warning message")]
    warning_message: Option<String>,
    interval: usize,
    #[serde(rename = "min interval")]
    min_interval: Option<usize>,
    #[serde(rename = "tracker id")]
    tracker_id: Option<String>,
    complete: Option<usize>,
    incomplete: Option<usize>,
    peers: AnnounceResponsePeers,
}

/// Address of a peer.
#[derive(Debug)]
pub struct PeerAddress {
    ip: IpAddr,
    port: u16,
}

/// Serialize [`PeerAddress`] as [`String`].
impl ToString for PeerAddress {
    fn to_string(&self) -> String {
        format!("{0}:{1}", self.ip(), self.port())
    }
}

impl PeerAddress {
    pub const fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl AnnounceResponse {
    /// Parse peers from [`AnnounceResponse`] as [`PeerAddress`].
    pub fn peers(&self) -> Result<Vec<PeerAddress>, Error> {
        match &self.peers {
            AnnounceResponsePeers::Raw(peers) => peers
                .iter()
                .map(|peer| {
                    let ip = peer
                        .ip
                        .parse::<IpAddr>()
                        .map_err(Error::IPAddressParseFailed)?;
                    Ok(PeerAddress::new(ip, peer.port))
                })
                .collect(),
            AnnounceResponsePeers::Compact(data) => data
                .chunks(COMPACT_PEER_ADDRESS_BYTES_LENGTH)
                .map(|parts| {
                    if parts.len() != COMPACT_PEER_ADDRESS_BYTES_LENGTH {
                        return Err(Error::PeerAddressInvalidLength(parts.len()));
                    }
                    let mut ip_bytes: [u8; COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH] =
                        [0; COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH];
                    ip_bytes.copy_from_slice(&parts[0..COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH]);
                    let ip = IpAddr::from(ip_bytes);
                    let port = Cursor::new(
                        &parts[COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH
                            ..COMPACT_PEER_ADDRESS_IP_BYTES_LENGTH
                                + COMPACT_PEER_ADDRESS_PORT_BYTES_LENGTH],
                    )
                    .read_u16::<BigEndian>()
                    .map_err(Error::IO)?;
                    Ok(PeerAddress::new(ip, port))
                })
                .collect(),
        }
    }
}

/// [`AnnounceResponse`] peers enum, that can be compact (binary)/non-compact (bencoded (https://wiki.theory.org/BitTorrentSpecification#Bencoding)).
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
#[serde(untagged)]
pub enum AnnounceResponsePeers {
    Raw(Vec<AnnounceResponsePeersRaw>),
    Compact(ByteBuf),
}

/// Bencoded (https://wiki.theory.org/BitTorrentSpecification#Bencoding) [`AnnounceResponse`] peers data.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
pub struct AnnounceResponsePeersRaw {
    #[serde(rename = "peer id")]
    peer_id: ByteBuf,
    ip: String,
    port: u16,
}
