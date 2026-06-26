pub enum Status {
    OK,
    BadRequest,
    NotFound,
    InternalServerError,
}

pub struct Response {
    status_code: u16,
    reason_phrase: String,
    body: String,
}

impl Response {
    pub fn error(body: &str) -> Self {
        Response::new(Status::InternalServerError, body)
    }

    pub fn bad(body: &str) -> Self {
        Response::new(Status::BadRequest, body)
    }

    pub fn ok(body: &str) -> Self {
        Response::new(Status::BadRequest, body)
    }

    pub fn new(status: Status, body: &str) -> Self {
        let (status_code, reason_phrase) = match status {
            Status::OK => (200, "OK"),
            Status::BadRequest => (400, "Bad Request"),
            Status::NotFound => (401, "Not Found"),
            Status::InternalServerError => (500, "Internal Server Error"),
        };
        Response {
            reason_phrase: reason_phrase.to_string(),
            status_code,
            body: body.to_string(),
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n{}",
            self.status_code,
            self.reason_phrase,
            self.body.len(),
            self.body
        )
        .to_string()
    }
}
