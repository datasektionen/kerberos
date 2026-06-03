use std::io::Write;

#[derive(Debug)]
pub enum RequestResult {
    Success,
    NotFound,
    RateLimited,
    NotMember(String),
    MeetingNotActive,
    TokenNotFound,
    OnboardConflict(String),
    Error(String),
}

impl RequestResult {
    fn from(response: Result<reqwest::blocking::Response, reqwest::Error>) -> Self {
        let response = match response {
            Ok(r) => r,
            Err(e) => return Self::Error(format!("Request error: {}", e)),
        };

        use reqwest::StatusCode as SC;
        match response.status() {
            SC::OK => Self::Success,
            SC::UNPROCESSABLE_ENTITY => Self::NotFound,
            SC::TOO_MANY_REQUESTS => Self::RateLimited,
            SC::UNAVAILABLE_FOR_LEGAL_REASONS => {
                let body = response
                    .text()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Self::NotMember(body)
            }
            SC::GONE => Self::MeetingNotActive,
            SC::UNAUTHORIZED => Self::TokenNotFound,
            SC::CONFLICT => {
                let body = response
                    .text()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Self::OnboardConflict(body)
            }
            s => {
                let body = response
                    .text()
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                Self::Error(format!("HTTP {}: {}", s, body))
            }
        }
    }
}

pub struct CardRequestBuilder<'a, 'b> {
    uid: String,
    server_url: &'b str,
    token: Option<&'b str>,
    file: Option<&'a mut std::fs::File>,
    onboard: bool,
}

impl<'a, 'b> CardRequestBuilder<'a, 'b> {
    pub fn new(uid: String, server_url: &'b str) -> Self {
        Self {
            uid,
            server_url,
            token: None,
            file: None,
            onboard: false,
        }
    }

    pub fn token(mut self, token: &'b str) -> Self {
        self.token = Some(token);
        self
    }

    pub fn file(mut self, file: Option<&'a mut std::fs::File>) -> Self {
        self.file = file;
        self
    }

    pub fn onboard(mut self, onboard: bool) -> Self {
        self.onboard = onboard;
        self
    }

    pub fn build(self, client: &reqwest::blocking::Client) -> RequestResult {
        let url = if self.onboard {
            format!("{}/onboard", self.server_url)
        } else {
            format!("{}/card", self.server_url)
        };

        let body = if self.onboard {
            println!("Enter the kthid for the card (e.g. \"turetek\"): ");
            std::io::stdout().flush().expect("Failed to flush stdout");
            let mut buf = String::new();
            std::io::stdin()
                .read_line(&mut buf)
                .expect("Failed to read input from stdin");
            let kthid = buf.trim();
            format!("{}#{}", kthid, self.uid)
        } else {
            self.uid.clone()
        };

        if let Some(file) = self.file {
            match writeln!(file, "{}", body) {
                Ok(()) => log::info!("Wrote card UID to file: {}", self.uid),
                Err(e) => log::error!("Writing to file: {e}"),
            };
        }

        let response = client
            .post(url)
            .bearer_auth(self.token.unwrap_or_default())
            .body(body)
            .send();

        RequestResult::from(response)
    }
}
