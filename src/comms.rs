use std::io;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use rustls::ClientConfig;
use rustls::ClientSession;
use rustls::Stream;
use webpki::DNSNameRef;
use webpki_roots;

use errors::*;

pub struct Comm {
    sess: ClientSession,
    sock: TcpStream,
    buf: Vec<u8>,
}

impl Comm {
    pub fn connect(hostname: &str, port: u16) -> Result<Comm> {
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let dns_name = DNSNameRef::try_from_ascii_str(hostname).unwrap();
        let sess = ClientSession::new(&Arc::new(config), dns_name);
        let sock = TcpStream::connect((hostname, port)).unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(1)))?;
        Ok(Comm {
            sess,
            sock,
            buf: Vec::with_capacity(200),
        })
    }

    fn stream(&mut self) -> Stream<ClientSession, TcpStream> {
        Stream::new(&mut self.sess, &mut self.sock)
    }

    pub fn write_line<S: AsRef<str>>(&mut self, line: S) -> Result<()> {
        let line = line.as_ref();
        println!("-> {}", line);
        if line.contains('\n') {
            bail!("Bad message: {:?}", line);
        }

        let mut stream = self.stream();
        stream.write_all(line.as_bytes())?;
        stream.write_all(b"\r\n")?;

        Ok(())
    }

    pub fn read_line(&mut self) -> Result<Option<String>> {
        loop {
            if let Some(first_newline) = self.buf.iter().position(|&x| x == b'\n') {
                let ret = String::from_utf8_lossy(&self.buf[..first_newline])
                    .trim()
                    .to_string();
                self.buf = self.buf[first_newline + 1..].to_vec();
                return Ok(Some(ret));
            }

            {
                let mut bytes = [0u8; 4096];
                let read = match self.stream().read(&mut bytes) {
                    Ok(read) => read,
                    Err(e) => if io::ErrorKind::WouldBlock == e.kind()
                        || io::ErrorKind::TimedOut == e.kind()
                    {
                        return Ok(None);
                    } else {
                        bail!(e)
                    },
                };
                self.buf.extend(&bytes[..read]);
            }
        }
    }
}
