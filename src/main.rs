use std::env;

mod bencode;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = bencode::decode_bencoded_value(encoded_value);

        let x = match decoded_value {
            serde_bencode::value::Value::Int(x) => format!("{}", x),
            serde_bencode::value::Value::Bytes(v) => format!("{}", std::str::from_utf8(&v).unwrap()),
            _ => panic!("Lol"),
        };

        println!("{}", x);
    } else {
        println!("unknown command: {}", args[1])
    }
}
