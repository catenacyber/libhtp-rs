#![allow(non_snake_case)]
#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate htp;
extern crate chrono;

use chrono::{DateTime, Utc};
use htp::{
    bstr::Bstr,
    config::{Config, HtpServerPersonality},
    connection_parser::{ConnectionParser, HtpStreamState},
    error::Result,
    transaction::{
        Data,
    },
};
use std::{
    convert::TryInto,
    env,
    iter::IntoIterator,
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    time::SystemTime,
};

#[derive(Debug)]
enum Chunk {
    Client(Vec<u8>),
    Server(Vec<u8>),
}

struct MainUserData {
    pub request_data: Vec<Bstr>,
    pub response_data: Vec<Bstr>,
}

impl MainUserData {
    pub fn new() -> Self {
        Self {
            request_data: Vec::with_capacity(5),
            response_data: Vec::with_capacity(5),
        }
    }
}

#[derive(Debug)]
struct TestInput {
    chunks: Vec<Chunk>,
}

impl IntoIterator for TestInput {
    type Item = Chunk;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.chunks.into_iter()
    }
}

impl TestInput {
    fn new_aux(input: &[u8]) -> Self {
        let mut test_input = TestInput { chunks: Vec::new() };
        let mut current = Vec::<u8>::new();
        let mut client = true;
        for line in input.split(|c| *c == b'\n') {
            if line.len() >= 3
                && ((line[0] == b'>' && line[1] == b'>' && line[2] == b'>')
                    || (line[0] == b'<' && line[1] == b'<' && line[2] == b'<'))
            {
                if !current.is_empty() {
                    // Pop off the CRLF from the last line, which
                    // just separates the previous data from the
                    // boundary <<< >>> chars and isn't actual data
                    if let Some(b'\n') = current.last() {
                        current.pop();
                    }
                    if let Some(b'\r') = current.last() {
                        current.pop();
                    }
                    test_input.append(client, current);
                    current = Vec::<u8>::new();
                }
                client = line[0] == b'>';
            } else {
                current.append(&mut line.to_vec());
                current.push(b'\n');
            }
        }
        // Remove the '\n' we would have appended for EOF
        current.pop();
        test_input.append(client, current);
        test_input
    }
    fn new(file: PathBuf) -> Self {
        let input = std::fs::read(file);
        assert!(input.is_ok());
        let input = input.unwrap();
        return TestInput::new_aux(&input);
    }

    fn append(&mut self, client: bool, data: Vec<u8>) {
        if client {
            self.chunks.push(Chunk::Client(data));
        } else {
            self.chunks.push(Chunk::Server(data));
        }
    }
}

#[derive(Debug)]
pub enum TestError {
    //MultipleClientChunks,
    //MultipleServerChunks,
    StreamError,
}

pub struct Test {
    connp: ConnectionParser,
    basedir: PathBuf,
}

fn TestConfig() -> Config {
    let mut cfg = Config::default();
    cfg.set_server_personality(HtpServerPersonality::APACHE_2)
        .unwrap();
    // The default bomb limit may be slow in some development environments causing tests to fail.
    cfg.compression_options.set_time_limit(std::u32::MAX);
    cfg.set_parse_urlencoded(true);
    cfg.set_parse_multipart(true);

    return cfg;
}

impl Test {
    fn new(cfg: Config) -> Self {
        let basedir = if let Ok(dir) = std::env::var("srcdir") {
            PathBuf::from(dir)
        } else {
            let mut base = PathBuf::from(
                env::var("CARGO_MANIFEST_DIR").expect("Could not determine test file directory"),
            );
            base.push("tests");
            base.push("files");
            base
        };

        let connp = ConnectionParser::new(cfg);
        Test { connp, basedir }
    }
    fn new_with_callbacks() -> Self {
        let mut cfg = TestConfig();
        cfg.register_response_body_data(response_body_data);
        cfg.register_request_body_data(request_body_data);
        let mut t = Test::new(cfg);
        // Configure user data and callbacks
        t.connp
            .response_mut()
            .set_user_data(Box::new(MainUserData::new()));
        t
    }
    fn run_aux(&mut self, test: TestInput) -> std::result::Result<(), TestError> {
        let tv_start = DateTime::<Utc>::from(SystemTime::now());
        self.connp.open(
            Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            Some(10000),
            Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            Some(80),
            Some(tv_start),
        );

        let mut request_buf: Option<Vec<u8>> = None;
        let mut response_buf: Option<Vec<u8>> = None;
        for chunk in test {
            match chunk {
                Chunk::Client(data) => {
                    let rc = self
                        .connp
                        .request_data(data.as_slice().into(), Some(tv_start));

                    if rc == HtpStreamState::ERROR {
                        return Err(TestError::StreamError);
                    }

                    if rc == HtpStreamState::DATA_OTHER {
                        let consumed = self
                            .connp
                            .request_data_consumed()
                            .try_into()
                            .expect("Error retrieving number of consumed bytes.");
                        let mut remaining = Vec::with_capacity(data.len() - consumed);
                        remaining.extend_from_slice(&data[consumed..]);
                        request_buf = Some(remaining);
                    }
                }
                Chunk::Server(data) => {
                    // If we have leftover data from before then use it first
                    if let Some(ref response_remaining) = response_buf {
                        let rc = (&mut self.connp)
                            .response_data(response_remaining.into(), Some(tv_start));
                        response_buf = None;
                        if rc == HtpStreamState::ERROR {
                            return Err(TestError::StreamError);
                        }
                    }

                    // Now use up this data chunk
                    let rc =
                        (&mut self.connp).response_data(data.as_slice().into(), Some(tv_start));
                    if rc == HtpStreamState::ERROR {
                        return Err(TestError::StreamError);
                    }

                    if rc == HtpStreamState::DATA_OTHER {
                        let consumed = self
                            .connp
                            .response_data_consumed()
                            .try_into()
                            .expect("Error retrieving number of consumed bytes.");
                        let mut remaining = Vec::with_capacity(data.len() - consumed);
                        remaining.extend_from_slice(&data[consumed..]);
                        response_buf = Some(remaining);
                    }

                    // And check if we also had some input data buffered
                    if let Some(ref request_remaining) = request_buf {
                        let rc = self
                            .connp
                            .request_data(request_remaining.into(), Some(tv_start));
                        request_buf = None;
                        if rc == HtpStreamState::ERROR {
                            return Err(TestError::StreamError);
                        }
                    }
                }
            }
        }

        // Clean up any remaining server data
        if let Some(ref response_remaining) = response_buf {
            let rc = (&mut self.connp).response_data(response_remaining.into(), Some(tv_start));
            if rc == HtpStreamState::ERROR {
                return Err(TestError::StreamError);
            }
        }
        self.connp
            .close(Some(DateTime::<Utc>::from(SystemTime::now())));
        Ok(())
    }
    fn run(&mut self, file: &str) -> std::result::Result<(), TestError> {
        let mut path = self.basedir.clone();
        path.push(file);
        let test = TestInput::new(path);
        return self.run_aux(test);
    }
    pub fn run_slice(&mut self, data: &[u8]) -> std::result::Result<(), TestError> {
        let test = TestInput::new_aux(data);
        return self.run_aux(test);
    }
}

fn response_body_data(d: &mut Data) -> Result<()> {
    let user_data = unsafe { (*d.tx()).user_data_mut::<MainUserData>().unwrap() };
    user_data
        .response_data
        .push(Bstr::from(d.as_slice().unwrap()));
    Ok(())
}

fn request_body_data(d: &mut Data) -> Result<()> {
    let user_data = unsafe { (*d.tx()).user_data_mut::<MainUserData>().unwrap() };
    user_data
        .request_data
        .push(Bstr::from(d.as_slice().unwrap()));
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let mut t = Test::new(TestConfig());
    t.run_slice(data);
});
