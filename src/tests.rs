#[cfg(test)]
mod tests {
    use crate::client::Client;

    #[test]
    fn test_create_client() {
        Client::new("0123456789".to_owned());
    }
}
