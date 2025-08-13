use std::net::{Ipv4Addr, SocketAddrV4};

use hickory_resolver::{IntoName, Resolver};

pub async fn resolve_mc(addr: String, port: u16) -> SocketAddrV4 {
    let resolver = Resolver::builder_tokio().unwrap().build();
    if port == 25565 {
        let result = resolver
            .srv_lookup(format!("_minecraft._tcp.{addr}"))
            .await
            .unwrap();
        let lookup_result = result.iter().next().unwrap();
        let target = lookup_result.target();

        let result = resolver.ipv4_lookup(target.to_string()).await.unwrap();
        let lookup_result = result.iter().next().unwrap();
        let ip = lookup_result.to_ip().unwrap();
        SocketAddrV4::new(ip.to_string().parse::<Ipv4Addr>().unwrap(), port)
    } else {
        let result = resolver.ipv4_lookup(addr.to_string()).await.unwrap();
        let lookup_result = result.iter().next().unwrap();
        let ip = lookup_result.to_ip().unwrap();
        SocketAddrV4::new(ip.to_string().parse::<Ipv4Addr>().unwrap(), port)
    }
}
