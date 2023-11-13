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

pub fn calculate_info_hash(
    torrent_info: TorrentInfo,
) -> Result<String, Box<dyn std::error::Error>> {
    let serialized = serde_bencode::to_bytes(&torrent_info)?;

    let mut hasher = Sha1::new();
    hasher.update(serialized);

    return Ok(format!("{:x}", hasher.finalize()));
}
