use std::{
    fs,
    net::{Ipv4Addr, SocketAddrV4},
};

pub fn get_proxies() -> Vec<SocketAddrV4> {
    let content = fs::read_to_string("proxies.txt").expect("couldn't find proxies.txt");
    let list: Vec<String> = content.lines().map(String::from).collect();
    let mut proxies: Vec<SocketAddrV4> = Vec::new();
    for proxy in list {
        let mut split = proxy.split(":");
        let server_address = split
            .next()
            .expect("failed to get a proxy from proxies.txt")
            .parse::<Ipv4Addr>()
            .expect("couldn't parse target as ipv4 addr");
        let server_port = split
            .next()
            .expect("failed to get a port from proxies.txt")
            .parse::<u16>()
            .expect("couldn't parse port as an u16");

        proxies.push(SocketAddrV4::new(server_address, server_port));
    }
    proxies
}
