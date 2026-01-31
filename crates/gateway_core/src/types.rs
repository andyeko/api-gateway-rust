#[derive(Debug, Clone)]
pub struct Request {
    pub path: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            headers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            body: body.into(),
        }
    }

    pub fn unauthorized(body: impl Into<String>) -> Self {
        Self {
            status: 401,
            body: body.into(),
        }
    }
}
