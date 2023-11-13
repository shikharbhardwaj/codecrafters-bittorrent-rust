use bytes::{Bytes, BytesMut, BufMut};
use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};

use crate::{domain::{Torrent, calculate_info_hash, PeerInfo}, bencode::decode_announce_response};


pub struct Client {
    peer_id: String
}

impl Client {
    pub fn new(peer_id: String) -> Client {
        Client {
            peer_id
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

    pub async fn peer_handshake(&self, peer_addr: &String, torrent: &Torrent) -> Result<PeerInfo, Box<dyn std::error::Error>> {
        let mut stream = TcpStream::connect(peer_addr).await.expect("Failed to connect to peer.");

        let info_hash_hex = calculate_info_hash(&torrent.info)?;
        let decoded_info_hash = hex::decode(info_hash_hex).expect("Could not decode info hash");
        let message = self.get_handshake_message(&decoded_info_hash);

        stream.write_all(&message).await.expect("Failed to send handshake message to stream.");

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).await.expect("Failed to read peer reply from stream.");

        return Ok(PeerInfo::from_bytes(Bytes::from(buffer.to_vec()), &decoded_info_hash).expect("Could not parse peer response."))
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
}
