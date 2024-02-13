use anyhow::Context;
use std::path::PathBuf;
use std::{env, fs};
use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};

use http_server_starter_rust::request::HttpRequest;
use http_server_starter_rust::response;
use http_server_starter_rust::thread_pool::ThreadPool;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let served_directory = match args.get(2) {
        Some(s) => Some(s.clone()),
        None => None,
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let served_directory = served_directory.clone();
                pool.execute(move || {
                    if handle_connection(stream, served_directory).is_err() {
                        println!("fail to handle this connection");
                    };
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    anyhow::Ok(())
}

fn handle_connection(
    mut stream: TcpStream,
    served_directory: Option<String>,
) -> anyhow::Result<()> {
    let mut reader = BufReader::new(&stream);
    let request = HttpRequest::new(&mut reader).context("parse HTTP request")?;

    match &request.path[..] {
        "/" => response::respond_with_200_ok(&mut stream)?,
        s if s.starts_with("/echo/") => {
            let random_string = &s["/echo/".len()..];
            response::respond_with_text_content(&mut stream, random_string)
                .context("echo with input string")?;
        }
        s if s.starts_with("/user-agent") => {
            response::respond_with_text_content(&mut stream, &request.user_agent)
                .context("respond with user agent")?;
        }
        s if s.starts_with("/files/") => {
            let served_directory = served_directory.context("get served directory")?;
            let filename = &s["/files/".len()..];
            let mut path = PathBuf::new();
            path.push(served_directory);
            path.push(filename);
            if let Ok(file) = fs::read(path.as_path()) {
                response::respond_with_octet_stream(&mut stream, &file)
                    .context("respond with file")?;
            } else {
                response::respond_with_404_not_found(&mut stream)?
            };
        }
        _ => response::respond_with_404_not_found(&mut stream)?,
    }

    anyhow::Ok(())
}
