use urlencoding::encode;

#[derive(Debug, Clone)]
pub enum Protocol {
    Http,
    Https,
}

#[derive(Debug)]
pub struct Url {
    protocol: Protocol,
    domain: String,
    path: String,
    params: Vec<(String, String)>,
}

impl Url {
    pub fn new() -> Self {
        Url {
            protocol: Protocol::Https,
            domain: String::new(),
            path: String::new(),
            params: Vec::new(),
        }
    }

    pub fn set_protocol(&self, protocol: Protocol) -> Self {
        Url {
            protocol,
            domain: self.domain.clone(),
            path: self.path.clone(),
            params: self.params.clone(),
        }
    }

    pub fn set_domain(&self, domain: &str) -> Self {
        Url {
            protocol: self.protocol.clone(),
            domain: domain.to_owned(),
            path: self.path.clone(),
            params: self.params.clone(),
        }
    }

    pub fn set_path(&self, path: &str) -> Self {
        Url {
            protocol: self.protocol.clone(),
            domain: self.domain.clone(),
            path: path.to_owned(),
            params: self.params.clone(),
        }
    }

    pub fn add_param(&self, key: &str, value: &str) -> Self {
        let mut params = self.params.clone();
        params.push((key.to_owned(), value.to_owned()));
        Url {
            protocol: self.protocol.clone(),
            domain: self.domain.clone(),
            path: self.path.clone(),
            params,
        }
    }

    pub fn build(&self) -> String {
        let mut url = String::new();
        match self.protocol {
            Protocol::Http => url.push_str("http://"),
            Protocol::Https => url.push_str("https://"),
        }
        url.push_str(&self.domain);
        url.push_str(&self.path);
        if !self.params.is_empty() {
            url.push('?');
            for (key, value) in &self.params {
                url.push_str(&encode(key));
                url.push('=');
                url.push_str(&encode(value));
                url.push('&');
            }
            url.pop();
        }
        url
    }
}
