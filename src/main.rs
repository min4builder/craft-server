mod world;
use byteorder::{ReadBytesExt, NetworkEndian};
use mio::net::TcpListener;
use mio::{Token, Poll, PollOpt, Ready, Events};
use world::Server;
use std::collections::HashMap;
use std::io::{self, ErrorKind, Read, Write};
use std::time::Instant;
use std::sync::{Arc, RwLock};

struct Queue<T> {
    stream: T,
    wqueue: Vec<u8>,
}

impl<T> Queue<T> {
    fn new(stream: T) -> Queue<T> {
        Queue {
            stream: stream,
            wqueue: Vec::new(),
        }
    }
}

impl<T: Write> Write for Queue<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.wqueue.extend_from_slice(buf);
        match self.stream.write(&self.wqueue) {
            Ok(n) => {
                self.wqueue.clear();
                Ok(n)
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => {
                /* do nothing, flush will be called later */
                Ok(buf.len())
            }
            Err(err) => Err(err),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.write(&self.wqueue).map(|_| {})
    }
}

impl<T: Read> Read for Queue<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

fn main() {
    const SERVER: Token = Token(0);

    let listener = TcpListener::bind(&"0.0.0.0:4080".parse().unwrap()).unwrap();
    let poll = Poll::new().unwrap();
    poll.register(&listener, SERVER, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();
    let mut events = Events::with_capacity(1024);

    let mut new_id = 1;

    let mut clients = HashMap::new();
    let mut server = Server::new();

    let mut now = Instant::now();

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            server.tick(now.elapsed().as_secs() as f32 + now.elapsed().subsec_millis() as f32 * 1e-3);
            now = Instant::now();
            match event.token() {
                SERVER => {
                    loop {
                        match listener.accept() {
                            Ok((stream, _)) => {
                                poll.register(&stream, Token(new_id), Ready::readable(), PollOpt::edge()).unwrap();
                                let s = Arc::new(RwLock::new(Queue::new(stream)));
                                clients.insert(new_id, Arc::clone(&s));
                                server.connect(s, new_id).unwrap();
                                println!("New client: {}", new_id);
                                new_id += 1;
                            }
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                break;
                            }
                            e => panic!("Error: {:?}", e),
                        }
                    }
                }
                Token(id) => {
                    if event.readiness().is_writable() {
                        clients.get_mut(&id).unwrap().write().unwrap().flush().unwrap();
                    }
                    if event.readiness().is_readable() {
                        loop {
                            let mut reader = clients.get_mut(&id).unwrap().write().unwrap();
                            let size = match reader.read_u32::<NetworkEndian>() {
                                Ok(n) => n as usize,
                                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof
                                           || e.kind() == ErrorKind::ConnectionReset => {
                                    drop(reader);
                                    server.disconnect(id).unwrap();
                                    clients.remove(&id);
                                    println!("Client {} disconnected", id);
                                    break;
                                }
                                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                    break;
                                }
                                e => panic!("Error: {:?}", e),
                            };
                            let mut buf = vec![0; size];
                            let mut read = 0;
                            loop {
                                match reader.read(&mut buf[read..]) {
                                    Ok(n) => {
                                        read += n;
                                        if read == size {
                                            drop(reader);
                                            server.process_message(id, &buf).unwrap();
                                            break;
                                        }
                                    }
                                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                        /* XXX */
                                        continue;
                                    }
                                    e => panic!("Error: {:?}", e),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

