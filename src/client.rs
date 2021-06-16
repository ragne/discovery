use std::time::Duration;

use crate::models::*;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use thiserror::Error;
use ureq::{Agent, AgentBuilder, Error as ureqError, Response, Transport};

const QUERY: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'#').add(b'<').add(b'>');
const PATH: &AsciiSet = &QUERY.add(b'?').add(b'`').add(b'{').add(b'}');
const NO_CONTENT: u16 = 204;

#[derive(Error, Debug)]
pub enum EurekaError {
    #[error("Network error {}", .0)]
    Network(Transport),
    #[error("Unexpected status code {}", .0)]
    Http(u16, Response),
    #[error("Unexpected state encountered: {0}")]
    UnexpectedState(String),
    #[error("Parsing error {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("App {0} not found in registry")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, EurekaError>;

impl From<ureqError> for EurekaError {
    fn from(err: ureqError) -> Self {
        match err {
            ureqError::Status(code, resp) => Self::Http(code, resp),
            ureqError::Transport(t) => Self::Network(t),
        }
    }
}

#[derive(Debug)]
pub struct EurekaClient {
    client: Agent,
    url: String,
}

impl EurekaClient {
    pub fn new(url: String) -> Self {
        Self {
            client: AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
            url,
        }
    }

    fn format_url(&self, segments: &[&str]) -> String {
        format!(
            "{}/{}",
            self.url,
            &segments
                .iter()
                .map(|s| utf8_percent_encode(s, PATH).to_string())
                .collect::<Vec<String>>()
                .join(&"/")
        )
    }

    pub fn register(&self, app_name: &str, app_data: &Instance) -> Result<()> {
        let res = ureq::post(&self.format_url(&[app_name]))
            .set("Accept", "application/json")
            .send_json(
                serde_json::to_value(RegisterRequest { instance: app_data })
                    .expect("Cannot serialize register request"),
            );

        match res {
            Ok(res) => match res.status() {
                NO_CONTENT => Ok(()),
                status => Err(EurekaError::Http(status, res)),
            },
            Err(e) => Err(EurekaError::from(e)),
        }
    }

    pub fn get_all(&self) -> Result<Applications> {
        Ok(self
            .client
            .get(&self.format_url(&[]))
            .set("Accept", "application/json")
            .call()?
            .into_json::<Applications>()?)
    }

    pub fn get(&self, app_name: &str) -> Result<Vec<Instance>> {
        self.get_all()?
            .applications
            .remove(app_name)
            .ok_or_else(|| EurekaError::NotFound(app_name.to_owned()))
    }
}
