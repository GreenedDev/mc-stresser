use std::net::Ipv4Addr;

pub fn get_args(args: Vec<String>) -> Option<((Ipv4Addr, u16), u32)> {
    if args.len() != 3 {
        println!("Usage: ./project targetaddr targetport threads");
        return None;
    }
    let server_address = &args[0]
        .parse::<Ipv4Addr>()
        .expect("cannot parse ip as ipv4");
    let server_port = args[1]
        .parse::<u16>()
        .expect("couldn't parse port as an u16");

    let threads = args[2]
        .parse::<u32>()
        .expect("couldn't parse threads count as an integer.");
    Some(((*server_address, server_port), threads))
}
