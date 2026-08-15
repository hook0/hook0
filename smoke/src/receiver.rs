//! Where the webhook the instance really sent is caught.
//!
//! A signature the harness computed and then verified would only prove that two copies of the same
//! arithmetic agree. So nothing here signs anything: this listens on a socket the output worker is
//! pointed at, keeps the first delivery exactly as it arrived — the bytes of the body, the header
//! names and values in order — and hands that to every client to verify with its own code.
//!
//! It is an HTTP server only as far as one delivery needs, and every dimension of what a peer may
//! cost is capped: how long it may take to arrive, how long a head may be, how many header lines
//! it may carry, and how many bytes of body are read off the socket.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::Error;

/// The most bytes of request head read before the peer is abandoned.
const MAX_HEAD_BYTES: usize = 16 * 1024;

/// The most header lines one delivery may carry.
const MAX_HEADER_LINES: usize = 64;

/// The most bytes of body read off the socket.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// How long one peer is given to finish saying what it came to say.
const PEER_TIMEOUT: Duration = Duration::from_secs(10);

/// How often the accept loop wakes to check whether it is out of time.
const POLL: Duration = Duration::from_millis(50);

/// One delivery, as it arrived.
#[derive(Debug, Clone)]
pub struct Delivery {
    /// Header names lowercased, values as delivered, in the order they arrived.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Delivery {
    /// The value delivered under that name, which the signature header is read out of.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(delivered, _)| delivered == name)
            .map(|(_, value)| value.as_str())
    }
}

/// A socket the instance can reach, waiting for one delivery.
pub struct Listening {
    pub port: u16,
    caught: Receiver<Delivery>,
}

/// Opens the socket and starts catching, on a port the operating system picks.
pub fn listen() -> Result<Listening, Error> {
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
        .map_err(|cause| Error::Receiver { cause })?;
    let port = listener
        .local_addr()
        .map_err(|cause| Error::Receiver { cause })?
        .port();
    listener
        .set_nonblocking(true)
        .map_err(|cause| Error::Receiver { cause })?;

    let (caught, catching) = sync_channel(1);
    thread::spawn(move || serve(listener, caught));

    Ok(Listening {
        port,
        caught: catching,
    })
}

impl Listening {
    /// The first delivery, or a refusal once `within` has run out.
    pub fn first(&self, within: Duration) -> Result<Delivery, Error> {
        match self.caught.recv_timeout(within) {
            Ok(delivery) => Ok(delivery),
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => {
                Err(Error::NoDelivery {
                    seconds: within.as_secs(),
                })
            }
        }
    }
}

/// Accepts until one delivery has been caught, or until the caller has stopped listening.
fn serve(listener: TcpListener, caught: SyncSender<Delivery>) {
    // The thread outlives no run: the deadline is the same order as the one the caller waits
    // under, so a socket nobody ever reaches is released rather than held for the process.
    let deadline = Instant::now() + Duration::from_secs(600);
    while Instant::now() < deadline {
        match listener.accept() {
            Ok((stream, _)) => {
                if let Some(delivery) = read_one(stream) {
                    let _ = caught.send(delivery);
                    return;
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(POLL),
            Err(_) => return,
        }
    }
}

/// Reads one request, answers it, and says what arrived.
fn read_one(mut stream: TcpStream) -> Option<Delivery> {
    let _ = stream.set_read_timeout(Some(PEER_TIMEOUT));
    let _ = stream.set_write_timeout(Some(PEER_TIMEOUT));

    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut head = 0usize;

    let mut request_line = String::new();
    read_line(&mut reader, &mut request_line, &mut head)?;

    let mut headers: Vec<(String, String)> = Vec::new();
    loop {
        let mut line = String::new();
        read_line(&mut reader, &mut line, &mut head)?;
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if headers.len() == MAX_HEADER_LINES {
            return None;
        }
        let (name, value) = line.split_once(':')?;
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
    }

    let length: usize = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse().ok())
        .unwrap_or(0);
    if length > MAX_BODY_BYTES {
        return None;
    }

    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).ok()?;

    let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    let _ = stream.flush();

    Some(Delivery { headers, body })
}

/// One line of head, refusing a peer that sends a head larger than the ceiling.
fn read_line(reader: &mut BufReader<TcpStream>, into: &mut String, head: &mut usize) -> Option<()> {
    let read = reader.read_line(into).ok()?;
    if read == 0 {
        return None;
    }
    *head += read;
    if *head > MAX_HEAD_BYTES {
        return None;
    }
    Some(())
}
