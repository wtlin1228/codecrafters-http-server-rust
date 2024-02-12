use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(&stream);
                let mut line = vec![];
                reader
                    .read_until(b'\n', &mut line)
                    .context("read until \\n")?;
                let request = HttpRequest::new(&line).context("parse HTTP request")?;
                match &request.path[..] {
                    "/" => respond_with_200_ok(&mut stream)?,
                    s if s.starts_with("/echo/") => {
                        let random_string = &s["/echo/".len()..];
                        respond_with_text_content(&mut stream, random_string)
                            .context("echo with input string")?;
                    }
                    _ => respond_with_404_not_found(&mut stream)?,
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    anyhow::Ok(())
}

fn respond_with_200_ok(stream: &mut TcpStream) -> anyhow::Result<()> {
    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write(response.as_bytes())?;
    stream.flush()?;
    anyhow::Ok(())
}

fn respond_with_text_content(stream: &mut TcpStream, text_content: &str) -> anyhow::Result<()> {
    stream.write("HTTP/1.1 200 OK\r\n".as_bytes())?;
    stream.write("Content-Type: text/plain\r\n".as_bytes())?;
    stream.write(format!("Content-Length: {}\r\n\r\n", text_content.len()).as_bytes())?;
    stream.write(format!("{}", text_content).as_bytes())?;
    stream.flush()?;
    anyhow::Ok(())
}

fn respond_with_404_not_found(stream: &mut TcpStream) -> anyhow::Result<()> {
    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
    stream.write(response.as_bytes())?;
    stream.flush()?;
    anyhow::Ok(())
}

#[allow(dead_code)]
struct HttpRequest {
    http_method: String,
    path: String,
    http_version: String,
}

impl HttpRequest {
    fn new(line: &Vec<u8>) -> anyhow::Result<Self> {
        let line = std::str::from_utf8(line).context("validate input as a valid utf8 string")?;
        let mut splitted_line = line.split(' ');
        Ok(Self {
            http_method: splitted_line.next().context("get HTTP method")?.to_string(),
            path: splitted_line.next().context("get path")?.to_string(),
            http_version: splitted_line
                .next()
                .context("get HTTP version")?
                .to_string(),
        })
    }
}
