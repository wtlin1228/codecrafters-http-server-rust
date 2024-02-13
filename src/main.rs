use anyhow::Context;
use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
};

use http_server_starter_rust::request::HttpRequest;
use http_server_starter_rust::ThreadPool;

fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => pool.execute(|| {
                if handle_connection(stream).is_err() {
                    println!("fail to handle this connection");
                };
            }),
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
