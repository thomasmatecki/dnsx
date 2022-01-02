mod byte_packet_buffer;
mod dns;

use byte_packet_buffer::BytePacketBuffer;
use dns::packet::Question;
use dns::{Packet, QueryType, ResultCode};

use clap::Parser;
use log::{debug, info};
use std::net::UdpSocket;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long, default_value_t = 2053)]
    port: u16,
}

fn lookup(qname: &str, qtype: QueryType) -> Result<Packet> {
    // Forward queries to Google's public DNS
    let server = ("127.0.0.53", 53);

    let socket = UdpSocket::bind(("0.0.0.0", 43210))?;

    let mut packet = Packet::new();

    packet.header.id = 6666;
    packet.header.questions = 1;
    packet.header.recursion_desired = true;
    packet
        .questions
        .push(Question::new(qname.to_string(), qtype));

    let mut req_buffer = BytePacketBuffer::new();
    packet.write(&mut req_buffer)?;
    socket.send_to(&req_buffer.buf[0..req_buffer.pos], server)?;

    let mut res_buffer = BytePacketBuffer::new();
    socket.recv_from(&mut res_buffer.buf)?;

    debug!("Upstream response buffer: {:?}", res_buffer);

    Packet::from_buffer(&mut res_buffer)
}
/// Handle a single incoming packet
fn handle_query(socket: &UdpSocket) -> Result<()> {
    // With a socket ready, we can go ahead and read a packet. This will
    // block until one is received.
    let mut req_buffer = BytePacketBuffer::new();

    // The `recv_from` function will write the data into the provided buffer,
    // and return the length of the data read as well as the source address.
    // We're not interested in the length, but we need to keep track of the
    // source in order to send our reply later on.
    let (_length, src_addr) = socket.recv_from(&mut req_buffer.buf)?;

    debug!("Request Buffer: {:?}", req_buffer);

    // Next, `Packet::from_buffer` is used to parse the raw bytes into
    // a `Packet`.
    let mut request_packet = Packet::from_buffer(&mut req_buffer)?;

    // Create and initialize the response packet
    let mut response_packet = Packet::new();
    response_packet.header.id = request_packet.header.id;
    response_packet.header.recursion_desired = true;
    response_packet.header.recursion_available = true;
    response_packet.header.response = true;

    // In the normal case, exactly one question is present
    if let Some(question) = request_packet.questions.pop() {
        debug!("Received query: {:?}", question);

        // Since all is set up and as expected, the query can be forwarded to the
        // target server. There's always the possibility that the query will
        // fail, in which case the `SERVFAIL` response code is set to indicate
        // as much to the client. If rather everything goes as planned, the
        // question and response records as copied into our response packet.
        if let Ok(result) = lookup(&question.name, question.qtype) {
            response_packet.questions.push(question);
            response_packet.header.rescode = result.header.rescode;
            response_packet.answers = result.answers;
            response_packet.authorities = result.authorities;
            response_packet.resources = result.resources;

            debug!("Answers: {:?}", response_packet.answers);
        } else {
            response_packet.header.rescode = ResultCode::SERVFAIL;
        }
    }
    // Being mindful of how unreliable input data from arbitrary senders can be, we
    // need make sure that a question is actually present. If not, we return `FORMERR`
    // to indicate that the sender made something wrong.
    else {
        response_packet.header.rescode = ResultCode::FORMERR;
    }

    // The only thing remaining is to encode our response and send it off!
    let mut res_buffer = BytePacketBuffer::new();

    debug!("Response Buffer: {:?}", req_buffer);
    response_packet.write(&mut res_buffer)?;

    let len = res_buffer.pos();
    let data = res_buffer.get_range(0, len)?;

    socket.send_to(data, src_addr)?;

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!("Listening on port {}", args.port);

    // Bind an UDP socket on port 2053
    let socket = UdpSocket::bind(("0.0.0.0", 2053))?;

    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
