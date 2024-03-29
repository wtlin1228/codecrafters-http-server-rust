use anyhow::Context;
use std::{
    io::{BufRead, BufReader, Read},
    net::TcpStream,
};

#[derive(Debug, Default)]
pub struct HttpRequest {
    pub http_method: String,
    pub path: String,
    pub http_version: String,
    pub host: String,
    pub user_agent: String,
    pub accept_encoding: String,
    pub content_length: usize,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(reader: &mut BufReader<&TcpStream>) -> anyhow::Result<Self> {
        let mut request = Self::default();

        // Parse the fist line, ex: GET /user-agent HTTP/1.1
        let mut first_line = vec![];
        reader
            .read_until(b'\n', &mut first_line)
            .context("read the first line from request stream")?;
        let mut splitted_line = std::str::from_utf8(&first_line).unwrap().split(' ');
        request.http_method = splitted_line.next().context("get HTTP method")?.to_string();
        request.path = splitted_line.next().context("get path")?.to_string();
        request.http_version = splitted_line
            .next()
            .context("get HTTP version")?
            .to_string();

        // Parse headers
        loop {
            let mut line = vec![];
            reader
                .read_until(b'\n', &mut line)
                .context("read one line from request stream")?;
            let line = std::str::from_utf8(&line).unwrap();
            match &line[..line.len() - 2] {
                l if l.is_empty() => {
                    break;
                }
                l if l.starts_with("Host: ") => {
                    request.host = l["Host: ".len()..].to_string();
                }
                l if l.starts_with("User-Agent: ") => {
                    request.user_agent = l["User-Agent: ".len()..].to_string();
                }
                l if l.starts_with("Accept-Encoding: ") => {
                    request.accept_encoding = l["Accept-Encoding: ".len()..].to_string();
                }
                l if l.starts_with("Content-Length: ") => {
                    request.content_length = l["Content-Length: ".len()..]
                        .parse::<usize>()
                        .context("parse content length")?;
                }
                _ => {
                    println!("unhandled request line: {:?}", line);
                }
            }
        }

        // Parse content
        if request.content_length > 0 {
            let mut buffer = Vec::with_capacity(request.content_length);
            buffer.resize_with(request.content_length, || 0);
            reader
                .read_exact(&mut buffer)
                .context("read request content")?;
            request.body = buffer;
        }

        Ok(request)
    }
}
