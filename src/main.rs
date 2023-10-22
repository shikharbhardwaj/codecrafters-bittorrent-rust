use std::env;

mod bencode;
mod domain;

use bencode::show_decoded_value;

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
        println!("Length: {:?}", decoded_torrent.info.length.unwrap())
    } else {
        println!("unknown command: {}", args[1])
    }
}
