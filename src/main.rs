use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream)?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    anyhow::Ok(())
}

fn handle_connection(mut stream: TcpStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(&stream);
    let request = HttpRequest::new(&mut reader).context("parse HTTP request")?;
    match &request.path[..] {
        "/" => respond_with_200_ok(&mut stream)?,
        s if s.starts_with("/echo/") => {
            let random_string = &s["/echo/".len()..];
            respond_with_text_content(&mut stream, random_string)
                .context("echo with input string")?;
        }
        s if s.starts_with("/user-agent") => {
            respond_with_text_content(&mut stream, &request.user_agent)
                .context("respond with user agent")?;
        }
        _ => respond_with_404_not_found(&mut stream)?,
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
#[derive(Debug, Default)]
struct HttpRequest {
    http_method: String,
    path: String,
    http_version: String,
    host: String,
    user_agent: String,
}

impl HttpRequest {
    fn new(reader: &mut BufReader<&TcpStream>) -> anyhow::Result<Self> {
        let mut request = Self::default();
        let mut lines_iter = reader
            .lines()
            .map(|l| l.context("read one line as string").unwrap());

        // GET /user-agent HTTP/1.1
        let line = lines_iter.next().context("read 1st line")?;
        let mut splitted_line = line.split(' ');
        request.http_method = splitted_line.next().context("get HTTP method")?.to_string();
        request.path = splitted_line.next().context("get path")?.to_string();
        request.http_version = splitted_line
            .next()
            .context("get HTTP version")?
            .to_string();

        // Host: localhost:4221
        let line = lines_iter.next().context("read 2nd line")?;
        let mut splitted_line = line.split(' ');
        request.host = splitted_line.nth(1).context("get host")?.to_string();

        // User-Agent: curl/7.64.1
        let line = lines_iter.next().context("read 3rd line")?;
        let mut splitted_line = line.split(' ');
        request.user_agent = splitted_line.nth(1).context("get user agent")?.to_string();

        Ok(request)
    }
}
