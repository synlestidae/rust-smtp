pub struct Response {
    pub code: u16,
    pub message: String,
    pub args: Option<Vec<String>>,
}

impl Response {
    pub fn new(code: u16, message: &str) -> Response {
        Response {
            code: code,
            message: message.to_string(),
            args: None,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!("{} {}\r\n", self.code, self.message).into_bytes()
    }
}
