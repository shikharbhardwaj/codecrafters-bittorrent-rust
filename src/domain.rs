use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

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

