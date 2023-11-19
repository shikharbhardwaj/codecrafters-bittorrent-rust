use std::env;

use bittorrent_starter_rust::{
    bencode::{show_decoded_value, decode_bencoded_value, decode_torrent}, 
    client::Client,
    domain::calculate_info_hash
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);

        show_decoded_value(decoded_value);
    } else if command == "info" {
        let file_path = &args[2];

        let decoded_torrent = decode_torrent(file_path).unwrap();

        println!("Tracker URL: {}", decoded_torrent.announce);
        println!("Length: {:?}", decoded_torrent.info.length.unwrap());
        println!(
            "Info Hash: {}",
            calculate_info_hash(&decoded_torrent.info).expect("Could not calculate info hash")
        );
        println!("Piece Length: {:?}", decoded_torrent.info.piece_length);

        let length = decoded_torrent.info.length.unwrap();
        let piece_length = decoded_torrent.info.piece_length;
        let mut num_pieces = length / piece_length;
        if length % piece_length != 0 {
            num_pieces += 1;
        }

        const SHA_LENGTH: usize = 20;
        let pieces = decoded_torrent.info.pieces;

        for i in 0..num_pieces {
            let start_idx = i as usize * SHA_LENGTH;
            let end_idx = (start_idx + SHA_LENGTH) as usize;
            println!("{:}", hex::encode(&pieces[start_idx..end_idx]));
        }
    } else if command == "peers" {
        let file_path = &args[2];

        let decoded_torrent = decode_torrent(file_path).unwrap();
        let client = Client::new("00112233445566778899".to_string());

        let peers = client
            .discover_peers(&decoded_torrent)
            .expect("Could not discover peers from torrent.");
        for peer in peers {
            println!("{}", peer);
        }
    } else if command == "handshake" {
        let file_path = &args[2];
        let peer_addr = &args[3];

        let decoded_torrent = decode_torrent(file_path).unwrap();
        let mut client = Client::new("00112233445566778899".to_string());

        let peer_info = client
            .peer_handshake(peer_addr, &decoded_torrent)
            .await
            .expect("Could not perform peer handshake");

        println!("Peer ID: {}", hex::encode(peer_info.id));
    } else {
        println!("unknown command: {}", args[1])
    }
}
