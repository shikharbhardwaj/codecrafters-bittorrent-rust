use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

extern crate sha1;

use sha1::{Digest, Sha1};
use tokio::{io::{BufReader, AsyncReadExt}, net::TcpStream};

use crate::debug;

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

impl Torrent {
    pub const BLOCK_SIZE: usize = 16 * 1024;

    pub fn get_num_pieces(&self) -> i64 {
        let length = self.info.length.unwrap();
        let piece_length = self.info.piece_length;
        let mut num_pieces = length / piece_length;
        if length % piece_length != 0 {
            num_pieces += 1;
        }

        return num_pieces;
    }

    pub fn get_num_blocks(&self) -> usize {
        self.info.piece_length as usize / Self::BLOCK_SIZE
    }

    pub fn get_block_length(&self, piece_index: usize, block_offset: usize) -> usize {
        match block_offset {
            _ if block_offset + 1 == self.get_num_blocks() && piece_index + 1 == self.get_num_pieces() as usize => {
                let last_piece_length = self.info.length.unwrap() % self.info.piece_length;
                last_piece_length as usize % Self::BLOCK_SIZE
            },
            _ => Self::BLOCK_SIZE,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct AnnounceResponse {
    pub interval: Option<i64>,
    pub peers: ByteBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PeerInfo {
    pub id: ByteBuf,
}

impl PeerInfo {
    pub fn from_bytes(
        input: Bytes,
        decoded_info_hash: &Vec<u8>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Check first byte (should be 19).
        let first_byte = input.get(0).expect("Did not find first byte in peer info");
        assert_eq!(*first_byte, b'\x13');

        // Check magic string "BitTorrent protocol".
        let magic_bytes = input
            .get(1..20)
            .expect("Did not find magic string in peer info");
        assert_eq!(magic_bytes, b"BitTorrent protocol");

        // Next 8 bytes are the zero bytes.
        // let zero_bytes = input.get(20..28).expect("Did not find zero bytes in peer info");
        // assert_eq!(zero_bytes, BytesMut::zeroed(8));

        let info_hash_bytes = input
            .get(28..48)
            .expect("Did not get info hash in peer info");
        assert_eq!(info_hash_bytes, decoded_info_hash);

        let peer_id_bytes = input
            .get(48..68)
            .expect("Did not find peer id in peer info");

        Ok(Self {
            id: ByteBuf::from(peer_id_bytes),
        })
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PeerMessage {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(i64),
    Bitfield(ByteBuf),
    Request(RequestMessage),
    Piece(PieceMessage),
    Cancel(RequestMessage),
    Keepalive,
}

impl PeerMessage {
    pub async fn from_stream(input: &mut BufReader<& mut TcpStream>) -> Result<Self, Box<dyn std::error::Error>> {
        // Get the message length.
        let message_length = input.read_u32().await?;

        if message_length == 0 {
            return Ok(PeerMessage::Keepalive);
        }

        let message_type = input 
            .read_u8().await?;

        match message_type {
            0 => Ok(PeerMessage::Choke),
            1 => Ok(PeerMessage::Unchoke),
            2 => Ok(PeerMessage::Interested),
            3 => Ok(PeerMessage::NotInterested),
            4 => {
                let have_idx = input.read_u32().await?.into();
                Ok(PeerMessage::Have(have_idx))
            },
            5 => {
                let mut payload = ByteBuf::with_capacity(message_length.try_into()?);
                input.read_exact(&mut payload).await?;
                Ok(PeerMessage::Bitfield(payload))
            },
            6 => {
                let index = input.read_u32().await?;
                let begin = input.read_u32().await?;
                let length = input.read_u32().await?;

                Ok(PeerMessage::Request(RequestMessage { index: index.into(), begin: begin.into(), length: length.into() }))
            },
            7 => {
                let index = input.read_u32().await?;
                let begin = input.read_u32().await?;

                let piece_length = message_length - 2 * 4 - 1;
                debug!("Reading piece data of length: {}", piece_length);
                let mut piece_data = vec![0; piece_length as usize];
                let bytes_read = input.read_exact(&mut piece_data).await?;

                Ok(PeerMessage::Piece(PieceMessage { index:  index.into(), begin: begin.into(), piece: piece_data }))
            },
            8 => {
                let index = input.read_u32().await?;
                let begin = input.read_u32().await?;
                let length = input.read_u32().await?;

                Ok(PeerMessage::Cancel(RequestMessage { index: index.into(), begin: begin.into(), length: length.into() }))
            },
            _ => Err(format!("Invalid peer message type: {}", message_type).into())
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            PeerMessage::Choke => b"\x00\x00\x00\x01\x00".to_vec(),
            PeerMessage::Unchoke => b"\x00\x00\x00\x01\x01".to_vec(),
            PeerMessage::Interested => b"\x00\x00\x00\x01\x02".to_vec(),
            PeerMessage::NotInterested => b"\x00\x00\x00\x01\x03".to_vec(),
            PeerMessage::Have(idx) => {
                let mut buf = vec![];
                let length: u32 = 5;
                buf.extend_from_slice(&length.to_be_bytes());
                buf.push(4);
                let idx: u32 = (*idx).try_into().expect("Could not convert HAVE idx to 4-byte integer");
                buf.extend_from_slice(&idx.to_be_bytes());
                buf
            },
            PeerMessage::Bitfield(bytes) => {
                let mut buf: Vec<u8> = vec![];

                let length: u32 = (bytes.len() + 1).try_into().expect("Could not convert Peer message length to 4-byte integer");
                buf.extend_from_slice(&length.to_be_bytes());
                buf.push(5);

                buf.extend(bytes.iter());
                buf
            },
            PeerMessage::Request(req) | PeerMessage::Cancel(req) => {
                let mut buf: Vec<u8> = vec![];
                let length: u32 = 3*4 + 1;
                buf.extend_from_slice(&length.to_be_bytes());
                buf.push(6);

                buf.extend(req.to_bytes());

                buf
            },
            PeerMessage::Piece(req) => {
                let mut buf: Vec<u8> = vec![];

                let length: u32 = 2*4 + 1 + req.piece.len() as u32;
                buf.extend_from_slice(&length.to_be_bytes());
                buf.push(6);

                buf.extend(req.to_bytes());

                buf
            },
            PeerMessage::Keepalive => {
                vec![0]
            },
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            PeerMessage::Choke => 0,
            PeerMessage::Unchoke => 1,
            PeerMessage::Interested => 2,
            PeerMessage::NotInterested => 3,
            PeerMessage::Have(_) => 4,
            PeerMessage::Bitfield(_) => 5,
            PeerMessage::Request(_) => 6,
            PeerMessage::Piece(_) => 7,
            PeerMessage::Cancel(_) => 8,
            _ => 9,
        }
    }
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct RequestMessage {
    pub index: u32,
    pub begin: u32,
    pub length: u32,
}

impl RequestMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend(self.index.to_be_bytes());
        buf.extend(self.begin.to_be_bytes());
        buf.extend(self.length.to_be_bytes());

        buf
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PieceMessage {
    pub index: u32,
    pub begin: u32,
    pub piece: Vec<u8>,
}

impl PieceMessage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend(self.index.to_be_bytes());
        buf.extend(self.begin.to_be_bytes());
        buf.extend(&self.piece);

        buf
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
