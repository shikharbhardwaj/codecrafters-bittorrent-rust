use std::env;

mod bencode;
mod domain;

use bencode::show_decoded_value;
use domain::calculate_info_hash;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = bencode::decode_bencoded_value(encoded_value);

        show_decoded_value(decoded_value);
    } else if command == "info" {
        let file_path = &args[2];

        let decoded_torrent = bencode::decode_torrent(file_path).unwrap();

        println!("Tracker URL: {}", decoded_torrent.announce);
        println!("Length: {:?}", decoded_torrent.info.length.unwrap());
        println!("Info Hash: {}", calculate_info_hash(&decoded_torrent.info).expect("Could not calculate info hash"));
        println!("Piece Length: {:?}", decoded_torrent.info.piece_length);

        let length = decoded_torrent.info.length.unwrap();
        let piece_length =  decoded_torrent.info.piece_length;
        let mut num_pieces = length / piece_length;
        if length % piece_length != 0 {
            num_pieces += 1;
        }

        const SHA_LENGTH:usize = 20;
        let pieces_vec = decoded_torrent.info.pieces.to_vec();

        for i in 0..num_pieces {
            let start_idx = i as usize * SHA_LENGTH;
            let end_idx = (start_idx + SHA_LENGTH) as usize;

            if start_idx < pieces_vec.len() && end_idx <= pieces_vec.len() && start_idx <= end_idx {
                let my_slice = &pieces_vec[start_idx..end_idx];

                // Print the resulting slice
                println!("{:}", hex::encode(my_slice))
            }
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
