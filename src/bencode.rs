use serde_bencode;

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
        _ => panic!("Lol"),
    };

    return x;
}

pub fn show_decoded_value(value: serde_bencode::value::Value) {
    println!("{}", decoded_value_to_string(&value));
}
