use crate::protocol::meta_info_file::{Sha1HashBytes, SHA1_HASH_BYTE_LENGTH};
use crate::protocol::transport::Transport;
use bytes::{BufMut, BytesMut};
use log::debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io;
use tokio::sync::Mutex;

/// Default protocol ID based on specification: https://wiki.theory.org/BitTorrentSpecification.
const DEFAULT_PROTOCOL_ID: &str = "BitTorrent protocol";

/// Errors from Peer Wire protocol (https://wiki.theory.org/BitTorrentSpecification#Peer_wire_protocol_.28TCP.29).
#[derive(Error, Debug)]
pub enum Error {
    #[error("connectivity error with peer")]
    ConnectionFailure(#[from] io::Error),
    #[error("empty handshake message")]
    EmptyHandshakeMessage,
    #[error("invalid handshake message bytes length")]
    InvalidHandshakeMessageBytesLength,
    #[error("invalid response handshake message: {0:?}")]
    InvalidResponseHandshake(HandshakeMessage),
    #[error("peer connection I/O timeout: {0:?}")]
    StreamIoTimeout(Duration),
}

/// Handshake message used to do handshake with peers.
#[derive(Debug)]
pub struct HandshakeMessage {
    peer_id: String,
    info_hash: Sha1HashBytes,
    protocol_id: String,
}

impl HandshakeMessage {
    /// Constructs new [`HandshakeMessage`] with optional `protocol_id` (default is [`DEFAULT_PROTOCOL_ID`]).
    pub fn new(peer_id: String, info_hash: Sha1HashBytes, protocol_id: Option<String>) -> Self {
        let mut protocol_id_final = DEFAULT_PROTOCOL_ID.to_string();
        if let Some(proto_id) = protocol_id {
            protocol_id_final = proto_id;
        }
        Self {
            peer_id,
            info_hash,
            protocol_id: protocol_id_final,
        }
    }
}

/// Serialize handshake message to bytes.
impl From<HandshakeMessage> for BytesMut {
    fn from(msg: HandshakeMessage) -> Self {
        let mut result = BytesMut::new();
        result.put_u8(msg.protocol_id.len() as u8);
        result.put_slice(msg.protocol_id.as_bytes());
        result.put_slice(&[0; 8]);
        result.put_slice(msg.info_hash.as_slice());
        result.put_slice(msg.peer_id.as_bytes());
        result
    }
}

/// Deserialize handshake message from bytes to struct.
impl TryFrom<Vec<u8>> for HandshakeMessage {
    type Error = Error;
    fn try_from(raw: Vec<u8>) -> Result<Self, Self::Error> {
        let protocol_id_length = raw.first().ok_or(Error::EmptyHandshakeMessage)?;
        let protocol_id_length = *protocol_id_length as usize;
        let message_size = protocol_id_length + 49; // from https://wiki.theory.org/BitTorrentSpecification#Handshake
        if raw.len() < message_size {
            return Err(Error::InvalidHandshakeMessageBytesLength);
        }
        let message = &raw[1..message_size];
        let protocol_id = &message[0..protocol_id_length];
        let info_hash: [u8; SHA1_HASH_BYTE_LENGTH] = message
            [protocol_id_length + 8..protocol_id_length + SHA1_HASH_BYTE_LENGTH + 8]
            .try_into()
            .map_err(|_| Error::InvalidHandshakeMessageBytesLength)?;
        let peer_id = &message[protocol_id_length + SHA1_HASH_BYTE_LENGTH + 8..];
        Ok(Self::new(
            String::from_utf8_lossy(peer_id).to_string(),
            info_hash,
            Some(String::from_utf8_lossy(protocol_id).to_string()),
        ))
    }
}

/// A peer connection wrapper, that should contain a transport implementation (see: [`Transport`])
/// to perform Peer Wire protocol (https://wiki.theory.org/BitTorrentSpecification#Peer_wire_protocol_.28TCP.29) operations.
pub struct PeerConnection<S>
where
    S: Transport,
{
    stream: Arc<Mutex<S>>,
    io_timeout: Duration,
}

impl<T: Transport> PeerConnection<T> {
    pub fn new(stream: T, io_timeout: Duration) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
            io_timeout,
        }
    }
    pub fn stream(&self) -> Arc<Mutex<T>> {
        self.stream.clone()
    }

    /// Send serialized handshake request to peer.
    async fn send_handshake_request(&self, message: &[u8]) -> Result<(), Error> {
        let stream = self.stream.lock().await;
        let peer = stream.peer_addr().unwrap();
        debug!(
            "[{0}:{1}] start handshake with peer...",
            peer.ip(),
            peer.port()
        );

        // send handshake message immediately
        loop {
            stream.writable().await.map_err(Error::ConnectionFailure)?;
            match stream.try_write(message) {
                Ok(_) => {
                    break;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(error) => {
                    return Err(Error::ConnectionFailure(error));
                }
            }
        }

        Ok(())
    }

    /// Read handshake message from the live peer connection.
    /// Important: [`Self::send_handshake_request`] must be called before reading from connection.
    async fn read_handshake_response(&self, info_hash: Sha1HashBytes) -> Result<(), Error> {
        let mut stream = self.stream.lock().await;
        let peer = stream.peer_addr().unwrap();

        let mut last_first_byte_zero_fail: Option<Instant> = None;
        // wait for handshake message from peer
        loop {
            // wait for handshake message's first byte to find out how many bytes we should read
            let mut buf = [0u8; 1];
            loop {
                stream.readable().await.map_err(Error::ConnectionFailure)?;
                match stream.try_read(&mut buf) {
                    Ok(_) => {
                        break;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(error) => {
                        return Err(Error::ConnectionFailure(error));
                    }
                }
            }

            if let Some(handshake_protocol_id_length) = buf.first() {
                let handshake_first_byte = *handshake_protocol_id_length;
                if handshake_first_byte == 0 {
                    match last_first_byte_zero_fail {
                        None => last_first_byte_zero_fail = Some(Instant::now()),
                        Some(last) => {
                            if last.elapsed() >= self.io_timeout {
                                return Err(Error::StreamIoTimeout(self.io_timeout));
                            } else {
                                continue;
                            }
                        }
                    }
                }

                // <protocol ID length> + 49 + 1 (first byte)
                let handshake_length = handshake_first_byte as usize + 50;

                // wait for handshake message's first byte
                let mut buf = Vec::with_capacity(handshake_length);
                buf.push(handshake_first_byte);
                stream
                    .read_buf(&mut buf)
                    .await
                    .map_err(Error::ConnectionFailure)?;
                if !buf.is_empty() {
                    let response_handshake = HandshakeMessage::try_from(buf)?;
                    debug!(
                        "[{0}:{1}] handshake response received: {2:?}",
                        peer.ip(),
                        peer.port(),
                        response_handshake
                    );

                    // validate
                    if response_handshake.info_hash.as_slice() != info_hash.as_slice() {
                        debug!("{0:?} != {1:?}", info_hash, response_handshake.info_hash);
                        return Err(Error::InvalidResponseHandshake(response_handshake));
                    }
                    debug!("[{0}:{1}] handshake is valid", peer.ip(), peer.port());

                    break;
                }
            }
        }

        Ok(())
    }

    /// Perform full handshake on a [`PeerConnection`].
    pub async fn handshake(
        &mut self,
        peer_id: String,
        info_hash: Sha1HashBytes,
    ) -> Result<(), Error> {
        let message: BytesMut = HandshakeMessage::new(peer_id.clone(), info_hash, None).into();
        tokio::time::timeout(
            self.io_timeout,
            self.send_handshake_request(message.as_ref()),
        )
        .await
        .map_err(|_| Error::StreamIoTimeout(self.io_timeout))??;
        tokio::time::timeout(self.io_timeout, self.read_handshake_response(info_hash))
            .await
            .map_err(|_| Error::StreamIoTimeout(self.io_timeout))??;

        Ok(())
    }
}
