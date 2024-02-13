use anyhow::Context;
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

#[derive(Debug, Default)]
pub struct HttpRequest {
    pub http_method: String,
    pub path: String,
    pub http_version: String,
    pub host: String,
    pub user_agent: String,
}

impl HttpRequest {
    pub fn new(reader: &mut BufReader<&TcpStream>) -> anyhow::Result<Self> {
        let mut request = Self::default();
        let mut lines_iter = reader
            .lines()
            .map(|l| l.context("read one line as string").unwrap());

        // ex: GET /user-agent HTTP/1.1
        let line = lines_iter.next().context("read 1st line")?;
        let mut splitted_line = line.split(' ');
        request.http_method = splitted_line.next().context("get HTTP method")?.to_string();
        request.path = splitted_line.next().context("get path")?.to_string();
        request.http_version = splitted_line
            .next()
            .context("get HTTP version")?
            .to_string();

        for line in lines_iter {
            match &line[..] {
                l if l.is_empty() => break,
                l if l.starts_with("Host: ") => {
                    request.host = line["Host: ".len()..].to_string();
                }
                l if l.starts_with("User-Agent: ") => {
                    request.user_agent = line["User-Agent: ".len()..].to_string();
                }
                _ => {
                    println!("unhandled request line: {:?}", line);
                }
            }
        }

        Ok(request)
    }
}
