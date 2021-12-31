pub mod packet;
pub mod query_type;
pub mod result_code;

pub use packet::Packet;

#[cfg(test)]
mod test {
    use super::*;
    use crate::byte_packet_buffer::BytePacketBuffer;
    use std::{fs::File, io::Read};

    #[test]
    fn test_request_packet() {
        let filename = "data/response_packet.txt";
        let mut file: File = File::open(filename)
            .unwrap_or_else(|why| panic!("couldn't open {} because {}", filename, why));

        let mut buffer = BytePacketBuffer::new();
        assert!(file.read(&mut buffer.buf).is_ok());

        let packet = Packet::from_buffer(&mut buffer).unwrap();

        assert_eq!(packet.header.id, 23034);
        assert_eq!(packet.header.recursion_desired, true);
        // TODO: more tests
    }
}
