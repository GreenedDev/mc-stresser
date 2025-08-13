use anyhow::{anyhow, Result};
use hickory_resolver::{name_server::ConnectionProvider, Resolver};
use std::net::{Ipv4Addr, SocketAddrV4};

async fn resolve_a<R: ConnectionProvider>(resolver: Resolver<R>, host: &str) -> Result<Ipv4Addr> {
    let response = resolver.ipv4_lookup(host).await.unwrap();
    if let Some(ip) = response.iter().next() {
        Ok(**ip)
    } else {
        Err(anyhow!("No A record found for {}", host))
    }
}

pub async fn resolve_mc(host: &str, port: u16, try_srv: bool) -> Result<SocketAddrV4> {
    let resolver = Resolver::builder_tokio().unwrap().build();

    if try_srv {
        if let Ok(srv_lookup) = resolver.srv_lookup(format!("_minecraft._tcp.{host}")).await {
            let srv_result = srv_lookup.iter().next().unwrap();

            if let Ok(ip) = resolve_a(resolver.clone(), &srv_result.target().to_string()).await {
                return Ok(SocketAddrV4::new(ip, srv_result.port()));
            }
        }
    }

    let ip = resolve_a(resolver.clone(), host).await?;
    Ok(SocketAddrV4::new(ip, port))
}

pub async fn parse_target(input: &str, default_port: u16) -> Result<SocketAddrV4> {
    // If user specified a port
    if let Some((host, port_str)) = input.rsplit_once(":") {
        let port = port_str.parse().map_err(|_| anyhow!("Invalid port"))?;

        if let Ok(ip) = host.parse::<Ipv4Addr>() {
            return Ok(SocketAddrV4::new(ip, port));
        }

        return resolve_mc(host, port, false).await;
    }

    // No port given, try SRV first
    return resolve_mc(input, default_port, true).await;
}
pub fn parse_hostname(input: &str) -> String {
    // If user specified a port
    if let Some((host, _)) = input.rsplit_once(":") {
        return host.to_string();
    }
    input.to_string()
}
