#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl Method {
    pub const ALL: [Method; 5] = [
        Method::Get,
        Method::Post,
        Method::Put,
        Method::Delete,
        Method::Patch,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Method::Get => Method::Post,
            Method::Post => Method::Put,
            Method::Put => Method::Delete,
            Method::Delete => Method::Patch,
            Method::Patch => Method::Get,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Method::Get => Method::Patch,
            Method::Post => Method::Get,
            Method::Put => Method::Post,
            Method::Delete => Method::Put,
            Method::Patch => Method::Delete,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Clone)]
pub struct RequestModel {
    pub method: Method,
    pub url: String,
    pub headers: Vec<KeyValue>,
    pub body: String,
    pub params: Vec<KeyValue>,
}

impl RequestModel {
    pub fn new() -> Self {
        Self {
            method: Method::Get,
            url: String::new(),
            headers: vec![KeyValue {
                key: String::new(),
                value: String::new(),
            }],
            body: String::new(),
            params: vec![KeyValue {
                key: String::new(),
                value: String::new(),
            }],
        }
    }
}
