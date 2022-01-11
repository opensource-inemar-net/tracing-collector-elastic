use reqwest::blocking::Client;
struct ElasticLogWriter {
    client: Client,
    user: String,
    password: String,
    url: String,
    appl: reqwest::header::HeaderValue,
}

impl ElasticLogWriter {
    #[allow(dead_code)]
    pub fn new(url: &str, user: &str, password: &str) -> ElasticLogWriter {
        ElasticLogWriter {
            client: Client::new(),
            user: user.to_string(),
            password: password.to_string(),
            url: url.to_string(),
            appl: reqwest::header::HeaderValue::from_static("application/json"),
        }
    }
}

impl std::io::Write for ElasticLogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let data = if let Ok(d) = String::from_utf8(buf.to_vec()) {
            d
        } else {
            return Ok(buf.len());
        };
        let res = self
            .client
            .post(self.url.clone())
            .basic_auth(self.user.clone(), Some(self.password.clone()))
            .header(reqwest::header::CONTENT_TYPE, self.appl.clone())
            .body(data)
            .send();
        match res {
            Ok(_) => Ok(buf.len()),
            Err(x) => Err(std::io::Error::new(std::io::ErrorKind::Other, x)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use mockito::{mock, server_address};

    #[test]
    /// The function is used to test the basic setup of mockito and sending a call to the system
    fn check_mock() {
        let _token = mock("POST", "/url/_doc").with_body(r#" "#).create();
        let server = format!("http://{}/url/_doc", server_address().to_string());

        let client = reqwest::blocking::Client::new();
        let res= client.post(server)
        .basic_auth("SERVICE_USER".to_string(),Some("SERVICE_PASSWORD".to_string()))
        .header(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"))
        .body("{\"node\" : \"laptop\", \"post_date\" : \"2021-10-19T19:20:21\",   \"message\" : \"Hello World 1\"}")
        .send();
        println!("Result {:?}", res);
    }

    #[test]
    fn elastic1() {
        use tracing::*;
        use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

        let _token = mock("POST", "/url/_doc").with_body(r#" "#).create();
        let server = format!("http://{}/url/_doc", server_address().to_string());

        let elastic = crate::ElasticLogWriter::new(&server, "DEMO_USER", "DEMO_PASSWORD");
        let (non_blocking, _guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
            .lossy(true)
            .buffered_lines_limit(2000)
            .finish(elastic);
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(tracing::Level::TRACE.into()))
            .with(
                fmt::Layer::new()
                    .json()
                    .flatten_event(true)
                    .with_writer(non_blocking),
            );
        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global collector");
        info!("Hello World");
    }
}
