
use std::net::TcpStream;
use std::io::{Read, Write};
use std::io::Result;

#[cfg(feature = "ssl")] use openssl::ssl::SslStream;

pub enum NetStream {
    PlainNetStream(TcpStream),

    #[cfg(feature = "ssl")]
    SecureNetStream(SslStream<TcpStream>)
}

impl Read for NetStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            &mut NetStream::PlainNetStream(ref mut stream) => stream.read(buf),
            #[cfg(feature = "ssl")]
            &mut NetStream::SecureNetStream(ref mut stream) => stream.read(buf),
        }
    }
}
impl Write for NetStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            &mut NetStream::PlainNetStream(ref mut stream) => stream.write(buf),
            #[cfg(feature = "ssl")]
            &mut NetStream::SecureNetStream(ref mut stream) => stream.write(buf),
        }
    }
    fn flush(&mut self) -> Result<()> {
        match self {
            &mut NetStream::PlainNetStream(ref mut stream) => stream.flush(),
            #[cfg(feature = "ssl")]
            &mut NetStream::SecureNetStream(ref mut stream) => stream.flush(),
        }
    }
}
