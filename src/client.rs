use crate::{domain::{Torrent, calculate_info_hash}, bencode::decode_announce_response};


pub struct Client {}

impl Client {
    pub fn new() -> Client {
        Client {  }
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

        let peer_id = "00112233445566778899";
        params.push(("peer_id", peer_id.to_string()));

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
}
