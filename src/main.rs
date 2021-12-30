mod byte_packet_buffer;
mod dns;

use byte_packet_buffer::BytePacketBuffer;
use dns::Packet;

use std::fs::File;
use std::io::Read;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

const FILENAME: &str = "data/response_packet.txt";

fn main() {
    let mut file: File = File::open(FILENAME)
        .unwrap_or_else(|why| panic!("couldn't open {} because {}", FILENAME, why));

    let mut buffer = BytePacketBuffer::new();
    let _read_ok = file.read(&mut buffer.buf);

    let packet = Packet::from_buffer(&mut buffer).unwrap();

    println!("{:#?}", packet);
}
