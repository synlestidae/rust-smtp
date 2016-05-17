pub struct Response {
    pub code: u16,
    pub message: &'static str,
    pub args: Option<Vec<String>>,
}

impl Response {
    pub fn new(code: u16, message: &'static str) -> Response {
        Response {
            code: code,
            message: message,
            args: None,
        }
    }
    pub fn new_with_args(code: u16, message: &'static str, args: Vec<String>) -> Response {
        Response {
            code: code,
            message: message,
            args: Some(args),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        format!("{} {}\r\n", self.code, self.message).into_bytes()
    }
}
