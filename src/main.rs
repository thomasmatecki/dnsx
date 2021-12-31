mod byte_packet_buffer;
mod dns;

use crate::dns::packet::Question;
use byte_packet_buffer::BytePacketBuffer;
use dns::query_type::QueryType;
use dns::Packet;
use rand::Rng;
use std::net::UdpSocket;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

fn main() -> Result<()> {
    // Perform an A query for google.com
    let mut rng = rand::thread_rng();

    let qname = "www.yahoo.com";

    // Using googles public DNS server
    let server = ("8.8.8.8", 53);

    // Bind a UDP socket to an arbitrary port
    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    // Build our query packet. It's important that we remember to set the
    // `recursion_desired` flag. As noted earlier, the packet id is arbitrary.
    let mut req_packet = Packet::new();

    req_packet.header.id = rng.gen();
    req_packet.header.recursion_desired = true;
    req_packet
        .questions
        .push(Question::new(qname.to_string(), QueryType::A));

    // Use our new write method to write the packet to a buffer...
    let mut req_buffer = BytePacketBuffer::new();
    req_packet.write(&mut req_buffer)?;

    println!("Request {:#?}", req_packet);

    // ...and send it off to the server using our socket:
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    // To prepare for receiving the response, we'll create a new `BytePacketBuffer`,
    // and ask the socket to write the response directly into our buffer.
    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;

    // As per the previous section, `DnsPacket::from_buffer()` is then used to
    // actually parse the packet after which we can print the response.
    let res_packet = Packet::from_buffer(&mut res_buffer)?;
    println!("Response {:#?}", res_packet);

    Ok(())
}
