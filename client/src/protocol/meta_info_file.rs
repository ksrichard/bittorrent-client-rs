use serde_bytes::ByteBuf;
use serde_derive::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use thiserror::Error;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncReadExt;

/// Standard size of SHA1 hashes in bytes from https://wiki.theory.org/BitTorrentSpecification.
pub const SHA1_HASH_BYTE_LENGTH: usize = 20;
pub type Sha1HashBytes = [u8; SHA1_HASH_BYTE_LENGTH];

/// Meta info (torrent file) related errors.
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read .torrent file")]
    FailedToReadFile(#[from] io::Error),
    #[error("failed to parse .torrent file")]
    FailedToParseFile(#[from] serde_bencode::Error),
    #[error("invalid number of bytes in info.pieces")]
    InvalidPiecesData,
}

/// Raw meta (torrent) file info base struct.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct RawMetaInfo {
    info: RawMetaInfoFile,
    announce: String,
}

/// Raw meta (torrent) file info.
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Hash)]
struct RawMetaInfoFile {
    pieces: ByteBuf,
    #[serde(rename = "piece length")]
    piece_length: isize,
    length: isize,
    name: String,
    #[serde(default)]
    md5sum: Option<String>,
}

impl RawMetaInfoFile {
    /// Parse file piece hashes (SHA-1) of meta info (torrent) file.
    pub fn parse_pieces(&self) -> Result<Vec<Sha1HashBytes>, Error> {
        if self.pieces.len() % SHA1_HASH_BYTE_LENGTH != 0 {
            return Err(Error::InvalidPiecesData);
        }
        Ok(self
            .pieces
            .chunks(SHA1_HASH_BYTE_LENGTH)
            .map(|hash| hash.to_vec().try_into().unwrap())
            .collect())
    }

    /// Returns SHA-1 hash of the whole info part.
    pub fn sha1_hash(&self) -> Result<Sha1HashBytes, Error> {
        let mut hasher = Sha1::default();
        let info_bytes = serde_bencode::to_bytes(self)?;
        let info_bytes = info_bytes.as_slice();
        hasher.update(info_bytes);
        let result: Sha1HashBytes = hasher.finalize().to_vec().try_into().unwrap();
        Ok(result)
    }
}

/// Parsed torrent file from [`RawMetaInfo`].
#[derive(Debug)]
pub struct TorrentFile {
    pub announce: String,
    pub info_hash: Sha1HashBytes,
    pub piece_hashes: Vec<Sha1HashBytes>,
    pub piece_length: isize,
    pub length: isize,
    pub name: String,
}

impl TorrentFile {
    pub const fn new(
        announce: String,
        info_hash: Sha1HashBytes,
        piece_hashes: Vec<Sha1HashBytes>,
        piece_length: isize,
        length: isize,
        name: String,
    ) -> Self {
        Self {
            announce,
            info_hash,
            piece_hashes,
            piece_length,
            length,
            name,
        }
    }
}

/// Convert [`RawMetaInfo`] to [`TorrentFile`].
impl TryFrom<RawMetaInfo> for TorrentFile {
    type Error = Error;

    fn try_from(raw: RawMetaInfo) -> Result<Self, Self::Error> {
        Ok(TorrentFile::new(
            raw.announce.clone(),
            raw.info.sha1_hash()?,
            raw.info.parse_pieces()?,
            raw.info.piece_length,
            raw.info.length,
            raw.info.name,
        ))
    }
}

/// Meta info file parser function.
pub async fn parse(file_path: &str) -> Result<TorrentFile, Error> {
    let mut file = File::open(file_path)
        .await
        .map_err(Error::FailedToReadFile)?;
    let mut content = vec![];
    file.read_to_end(&mut content)
        .await
        .map_err(Error::FailedToReadFile)?;
    let result: RawMetaInfo =
        serde_bencode::from_bytes(content.as_slice()).map_err(Error::FailedToParseFile)?;
    result.try_into()
}
