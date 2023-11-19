use std::collections::HashMap;

use bytes::{Bytes, BytesMut, BufMut};
use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt, BufReader}};

use crate::{
    domain::{Torrent, calculate_info_hash, PeerInfo, PeerMessage, RequestMessage},
    bencode::decode_announce_response, info, debug};

pub struct Client {
    peer_id: String,
    connections: HashMap<String, TcpStream>,
}

impl Client {
    pub fn new(peer_id: String) -> Client {
        let connections: HashMap<String, TcpStream> = HashMap::new();

        Client {
            peer_id,
            connections,
        }
    }

    pub fn discover_peers(&self, torrent: &Torrent) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut params = vec![];

        let mut announce_url = torrent.announce.clone();

        let info_hash = calculate_info_hash(&torrent.info)?;
        let decoded_info_hash = hex::decode(info_hash)?;
        let mut urlencoded_info_hash = "".to_string();

        for byte in decoded_info_hash {
            urlencoded_info_hash += "%";
            urlencoded_info_hash += &hex::encode(vec![byte]);
        }
        announce_url += &format!("?info_hash={}", urlencoded_info_hash);

        let peer_id = self.peer_id.clone();
        params.push(("peer_id", peer_id));

        let port = "6881";
        params.push(("port", port.to_string()));

        let uploaded = 0;
        params.push(("uploaded", uploaded.to_string()));

        let downloaded = 0;
        params.push(("downloaded", downloaded.to_string()));

        let left = torrent.info.length
            .expect("Did not find length in torrent info.");
        params.push(("left", left.to_string()));

        let compact = 1;
        params.push(("compact", compact.to_string()));

        let url_with_params = reqwest::Url::parse_with_params(&announce_url, params)?;

        let response = reqwest::blocking::get(url_with_params)?;
        let response_bytes = response.bytes()?;

        let decoded_response = decode_announce_response(&response_bytes);

        const PEER_SIZE: usize = 6;
        const IP_ADDR_SIZE: usize = 4;
        let peer_count = decoded_response.peers.len() / PEER_SIZE;

        let mut peers = vec![];

        for idx in 0..peer_count {
            let i = idx * PEER_SIZE;

            let mut peer_endpoint = vec![];

            for j in 0..IP_ADDR_SIZE {
                peer_endpoint.push(decoded_response.peers[i + j].to_string());

                if j != IP_ADDR_SIZE - 1 {
                    peer_endpoint.push(".".to_string());
                }
            }
            let port_byte0 = i + IP_ADDR_SIZE;
            let port_byte1 = port_byte0 + 1;

            let port = ((decoded_response.peers[port_byte0] as u16) << 8) | decoded_response.peers[port_byte1] as u16;
            peer_endpoint.push(":".to_string());
            peer_endpoint.push(port.to_string());

            peers.push(peer_endpoint.join(""));
        }

        return Ok(peers);
    }

    pub async fn peer_handshake(&mut self, peer_addr: &String, torrent: &Torrent) -> Result<PeerInfo, Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(peer_addr).await.expect("Failed to connect to peer.");

        let info_hash_hex = calculate_info_hash(&torrent.info)?;
        let decoded_info_hash = hex::decode(info_hash_hex).expect("Could not decode info hash");
        let message = self.get_handshake_message(&decoded_info_hash);

        stream.write_all(&message).await.expect("Failed to send handshake message to stream.");

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).await.expect("Failed to read peer reply from stream.");

        let peer_info = PeerInfo::from_bytes(Bytes::from(buffer.to_vec()), &decoded_info_hash).expect("Could not parse peer response.");

        let peer_id = hex::encode(&peer_info.id);

        self.connections.insert(peer_id, stream);

        return Ok(peer_info)
    }

    async fn recv_message(&mut self, peer_id: &String) -> Result<PeerMessage, Box<dyn std::error::Error>> {
        let stream = self.connections.get_mut(peer_id).expect("Did not find peer in active connections");
        let mut reader = BufReader::new(stream);

        PeerMessage::from_stream(& mut reader).await
    }

    async fn send_message(&mut self, peer_id: &String, message: &PeerMessage) -> Result<(), Box<dyn std::error::Error>> {
        let stream = self.connections.get_mut(peer_id).expect("Did not find peer in active connections");

        stream.write(&message.to_bytes()).await?;
        return Ok(())
    }

    fn get_handshake_message(&self, info_hash: &Vec<u8>) -> Bytes {
        let mut buf = BytesMut::with_capacity(1024);
        buf.put_u8(19);
        buf.put(&b"BitTorrent protocol"[..]);
        buf.put_bytes(0, 8);
        buf.put(info_hash.as_slice());
        buf.put(self.peer_id.as_bytes());

        return buf.into();
    }

    pub async fn download_piece(&mut self, piece_index: u32, torrent: &Torrent, peer_id: &String, output_path: &str) -> Result<(), Box<dyn std::error::Error>>{
        // 1. Get bitfield message.
        let bitfield_message = self.recv_message(peer_id).await?;
        match bitfield_message {
            PeerMessage::Bitfield(_) => {
                // TODO: Actually check if the bitfield message has the piece
                // index we've asked for, otherwise bail.
                info!("Received bitfield message from peer: {}", peer_id);
            },
            _ => return Err(format!("Invalid peer message type received instead of bitfield message: ").into())
        }

        // Say that we're interested in this peer.
        info!("Sending interested message to peer: {}", peer_id);
        self.send_message(peer_id, &PeerMessage::Interested).await?;
        info!("Sent interested message: {}", peer_id);
        let mut proceed = false;

        while !proceed {
            let unchoke_message = self.recv_message(peer_id).await?;

            match unchoke_message {
                PeerMessage::Unchoke => {
                    info!("Received unchoke message from peer: {}", peer_id);
                    proceed = true;
                },
                PeerMessage::Keepalive => {
                    info!("Received keepalive message from peer: {}", peer_id);
                },
                _ => return Err(format!("Invalid peer message type received instead of unchoke message: ").into())
            }
        }


        // We'll keep it simple and download the piece sequentially.
        let num_blocks = torrent.get_num_blocks();

        let mut piece_data: Vec<u8> = vec![];

        for block_offset in 0..num_blocks {
            debug!("Downloading block with offset: {}", block_offset);
            let request_message = RequestMessage{
                index: piece_index,
                begin: (block_offset * Torrent::BLOCK_SIZE) as u32,
                length: torrent.get_block_length(piece_index as usize, block_offset) as u32};
            
            self.send_message(peer_id, &PeerMessage::Request(request_message)).await?;
            debug!("Sent request message for block with offset: {}", block_offset);

            let mut block_pending= true;

            while block_pending {
                let message = self.recv_message(peer_id).await?;

                match message {
                    PeerMessage::Piece(piece) => {
                        debug!("Received peice message for block offset: {}", block_offset);
                        debug!("Got piece with index: {} begin: {}", piece.index, piece.begin);
                        piece_data.extend(&piece.piece);
                        block_pending = false;
                    },
                    _ => {}
                }
            }
        }

        // TODO:Calculate SHA to validate the piece 
        
        // Store piece in the output location.
        std::fs::write(output_path, piece_data)?;

        Ok(())
    }
}
