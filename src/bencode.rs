use serde_bencode;
use std::fs;

use crate::domain::Torrent;

pub fn decode_bencoded_value(encoded_value: &str) -> serde_bencode::value::Value {
    let deserialized: serde_bencode::value::Value = serde_bencode::from_str(&encoded_value).unwrap();

    return deserialized;
}

fn decoded_value_to_string(decoded_value: &serde_bencode::value::Value) -> String {
    let x = match decoded_value {
        serde_bencode::value::Value::Int(x) => format!("{}", x),
        serde_bencode::value::Value::Bytes(v) => format!("\"{}\"", std::str::from_utf8(v).unwrap()),
        serde_bencode::value::Value::List(v) => 
            format!("[{}]", v.iter().map(|x| decoded_value_to_string(x)).collect::<Vec<String>>().join(",")),
        serde_bencode::value::Value::Dict(v) => 
            format!("{{{}}}", v.iter().map(|x| format!("\"{}\":{}", std::str::from_utf8(x.0).unwrap(),
                decoded_value_to_string(x.1))).collect::<Vec<String>>().join(",")),
    };

    return x;
}

pub fn show_decoded_value(value: serde_bencode::value::Value) {
    println!("{}", decoded_value_to_string(&value));
}

pub fn decode_torrent(file_path: &str) -> Result<Torrent, &'static str> {
    let contents = fs::read(file_path).expect("Unable to read file contents");
    let torrent: Torrent = serde_bencode::from_bytes(&contents)
        .expect("Unable to parse bencoded torrent.");

    return Ok(torrent);
}
