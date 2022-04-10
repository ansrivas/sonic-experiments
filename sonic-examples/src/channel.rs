use sonic_channel::*;

pub struct Channel {
    pub url: String,
    pub password: String,
    pub username: Option<String>,
}

impl Channel {
    pub fn new<T>(url: T, username: T, password: T) -> Channel
    where
        T: Into<String>,
    {
        Channel {
            url: url.into(),
            password: password.into(),
            username: Some(username.into()),
        }
    }

    pub fn ingest(&self) -> IngestChannel {
        IngestChannel::start(self.url.clone(), self.password.clone()).unwrap()
    }

    pub fn search(&self) -> SearchChannel {
        SearchChannel::start(self.url.clone(), self.password.clone()).unwrap()
    }

    pub fn control(&self) -> ControlChannel {
        ControlChannel::start(self.url.clone(), self.password.clone()).unwrap()
    }
}
