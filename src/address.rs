#[derive(Debug)]
pub struct Address {
    pub address: String,
    pub domain: String,
}

impl Clone for Address {
    fn clone(&self) -> Address {
        Address {
            address: self.address.clone(),
            domain: self.domain.clone(),
        }
    }
}

impl Address {
    pub fn new(address: &str, domain: &str) -> Address {
        Address {
            address: address.to_string(),
            domain: domain.to_string(),
        }
    }
}
