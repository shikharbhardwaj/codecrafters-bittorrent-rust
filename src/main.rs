use std::{fs::File, io::BufWriter};

use bittorrent_starter_rust::{
    bencode::{show_decoded_value, decode_bencoded_value, decode_torrent}, 
    client::Client,
    domain::calculate_info_hash, info
};

use clap::{Command, Arg, ArgAction};

#[tokio::main]
async fn main() {
    let matches = Command::new("Your CLI App")
        .version("0.1.0")
        .author("shikharbhardwaj")
        .about("Codecrafters bittorrent starter rust")
        .subcommand(
            Command::new("decode")
                .about("Decode a string")
                .arg(Arg::new("input").index(1).required(true)),
        )
        .subcommand(
            Command::new("info")
                .about("Get information from a file")
                .arg(Arg::new("file_path").index(1).required(true)),
        )
        .subcommand(
            Command::new("peers")
                .about("Get peers from a file")
                .arg(Arg::new("file_path").index(1).required(true)),
        )
        .subcommand(
            Command::new("handshake")
                .about("Perform a handshake with a peer")
                .arg(Arg::new("file_path").index(1).required(true))
                .arg(Arg::new("peer_addr").index(2).required(true)),
        )
        .subcommand(
            Command::new("download_piece")
                .about("Download a piece with an output path")
                .arg(Arg::new("file_path").index(1).required(true))
                .arg(Arg::new("output_path").short('o').long("output").action(ArgAction::Set).required(true))
                .arg(Arg::new("piece_index").index(2).required(true).value_parser(clap::value_parser!(u32))),
        )
        .subcommand(
            Command::new("download")
                .about("Download the whole file")
                .arg(Arg::new("file_path").index(1).required(true))
                .arg(Arg::new("output_path").short('o').long("output").action(ArgAction::Set).required(true))
        )
        .get_matches();

    match matches.subcommand() {
        Some(("decode", sub_m)) => {
            // Handle decode subcommand
            let encoded_value: &String = sub_m.get_one("input").unwrap();

            let decoded_value = decode_bencoded_value(encoded_value);
            show_decoded_value(decoded_value);
        }
        Some(("info", sub_m)) => {
            // Handle info subcommand
            let file_path: &String= sub_m.get_one("file_path").unwrap();
            let decoded_torrent = decode_torrent(file_path).unwrap();

            println!("Tracker URL: {}", decoded_torrent.announce);
            println!("Length: {:?}", decoded_torrent.info.length.unwrap());
            println!(
                "Info Hash: {}",
                calculate_info_hash(&decoded_torrent.info).expect("Could not calculate info hash")
            );
            println!("Piece Length: {:?}", decoded_torrent.info.piece_length);

            let num_pieces = decoded_torrent.get_num_pieces();

            const SHA_LENGTH: usize = 20;
            let pieces = decoded_torrent.info.pieces;

            for i in 0..num_pieces {
                let start_idx = i as usize * SHA_LENGTH;
                let end_idx = (start_idx + SHA_LENGTH) as usize;
                println!("{:}", hex::encode(&pieces[start_idx..end_idx]));
            }
        }
        Some(("peers", sub_m)) => {
            // Handle peers subcommand
            let file_path:&String = sub_m.get_one("file_path").unwrap();

            let decoded_torrent = decode_torrent(file_path).unwrap();
            let client = Client::new("00112233445566778899".to_string());

            let peers = client
                .discover_peers(&decoded_torrent)
                .expect("Could not discover peers from torrent.");
            for peer in peers {
                println!("{}", peer);
            }
        }
        Some(("handshake", sub_m)) => {
            // Handle handshake subcommand
            let file_path: &String = sub_m.get_one("file_path").unwrap();
            let peer_addr = sub_m.get_one("peer_addr").unwrap();

            let decoded_torrent = decode_torrent(file_path).unwrap();
            let mut client = Client::new("00112233445566778899".to_string());

            let peer_info = client
                .peer_handshake(peer_addr, &decoded_torrent)
                .await
                .expect("Could not perform peer handshake");

            println!("Peer ID: {}", hex::encode(peer_info.id));
        }
        Some(("download_piece", sub_m)) => {
            // Handle download_piece subcommand
            let file_path: &String = sub_m.get_one("file_path").unwrap();
            let decoded_torrent = decode_torrent(file_path).unwrap();

            info!("Length: {:?}", decoded_torrent.info.length.unwrap());
            info!(
                "Info Hash: {}",
                calculate_info_hash(&decoded_torrent.info).expect("Could not calculate info hash")
            );
            info!("Piece Length: {:?}", decoded_torrent.info.piece_length);

            let output_path: &String = sub_m.get_one("output_path").unwrap();
            let piece_index: u32 = *sub_m.get_one("piece_index").unwrap();

            info!("Downloading piece index: {}", piece_index);

            let mut client = Client::new("00112233445566778899".to_string());

            // TODO: Make it query all peers.
            let peers = client.discover_peers(&decoded_torrent).expect("Could not discover peers from announce info");

            let peer_addr = &peers[0];
            info!("Initiating handshake with peer: {}", peer_addr);
            let peer_info = client
                .peer_handshake(peer_addr, &decoded_torrent)
                .await
                .expect("Could not perform peer handshake");
            let peer_id = hex::encode(peer_info.id);
            info!("Got peer id: {}", peer_id);

            let f = File::create(output_path).expect("Unable to create destination file.");
            let mut buf = BufWriter::new(f);

            client.download_piece(piece_index, &decoded_torrent, &peer_id, &mut buf).await.expect("Could not download piece");
        }
        Some(("download", sub_m)) => {
            let file_path: &String = sub_m.get_one("file_path").unwrap();
            let decoded_torrent = decode_torrent(file_path).unwrap();

            info!("Length: {:?}", decoded_torrent.info.length.unwrap());
            info!(
                "Info Hash: {}",
                calculate_info_hash(&decoded_torrent.info).expect("Could not calculate info hash")
            );
            info!("Piece Length: {:?}", decoded_torrent.info.piece_length);

            let output_path: &String = sub_m.get_one("output_path").unwrap();
            let mut client = Client::new("00112233445566778899".to_string());

            // TODO: Make it query all peers.
            let peers = client.discover_peers(&decoded_torrent).expect("Could not discover peers from announce info");

            let peer_addr = &peers[0];
            info!("Initiating handshake with peer: {}", peer_addr);
            let peer_info = client
                .peer_handshake(peer_addr, &decoded_torrent)
                .await
                .expect("Could not perform peer handshake");
            let peer_id = hex::encode(peer_info.id);
            info!("Got peer id: {}", peer_id);

            let f = File::create(output_path).expect("Unable to create destination file.");
            let mut buf = BufWriter::new(f);

            client.download_file(&decoded_torrent, &peer_id, &mut buf).await.expect("Could not download piece");
        }
        _ => {
            unreachable!("clap ensures we don't get here")
        }
    }
}