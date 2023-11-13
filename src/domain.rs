use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

extern crate sha1;

use sha1::{Digest, Sha1};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Torrent {
    pub announce: String,
    pub info: TorrentInfo,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TorrentInfo {
    pub length: Option<i64>,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    pub pieces: ByteBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct AnnounceResponse {
    pub interval: Option<i64>,
    pub peers: ByteBuf,
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PeerInfo {
    pub id: ByteBuf
}

impl PeerInfo {
    pub fn from_bytes(input: Bytes, decoded_info_hash: &Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        // Check first byte (should be 19).
        let first_byte = input.get(0).expect("Did not find first byte in peer info");
        assert_eq!(*first_byte, b'\x13');

        // Check magic string "BitTorrent protocol".
        let magic_bytes = input.get(1..20).expect("Did not find magic string in peer info");
        assert_eq!(magic_bytes, b"BitTorrent protocol");

        // Next 8 bytes are the zero bytes.
        // let zero_bytes = input.get(20..28).expect("Did not find zero bytes in peer info");
        // assert_eq!(zero_bytes, BytesMut::zeroed(8));

        let info_hash_bytes = input.get(28..48).expect("Did not get info hash in peer info");
        assert_eq!(info_hash_bytes, decoded_info_hash);

        let peer_id_bytes = input.get(48..68).expect("Did not find peer id in peer info");
        
        Ok(Self {
            id: ByteBuf::from(peer_id_bytes)
        })
    }
}

pub fn calculate_info_hash(
    torrent_info: &TorrentInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    let serialized = serde_bencode::to_bytes(&torrent_info)?;

    let mut hasher = Sha1::new();
    hasher.update(serialized);

    return Ok(format!("{:x}", hasher.finalize()));
}
