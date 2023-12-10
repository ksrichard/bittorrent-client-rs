//! BitTorrent client/peer library.
/// This a Bittorrent client/peer implementation that covers the basics of a fully featured client/peer.
///
/// Features:
/// - Read and parse .torrent files
/// - Communication with torrent tracker to get all bittorrent peers (only http ones for now)
/// - Connect to all peers parallel through TCP connection and perform handshake as initial steps of download process
///
/// The whole implementation is based on the BitTorrent specification: https://wiki.theory.org/BitTorrentSpecification
mod client;
mod protocol;

pub use client::*;
