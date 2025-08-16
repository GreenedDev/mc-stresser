use crate::methods::methods::{AttackMethod, method_to_port, method_to_srv_prefix};
use anyhow::{Result, anyhow};
use hickory_resolver::{Resolver, name_server::ConnectionProvider};
use std::net::{Ipv4Addr, SocketAddrV4};

async fn resolve_a<R: ConnectionProvider>(resolver: Resolver<R>, host: &str) -> Result<Ipv4Addr> {
	let response = resolver.ipv4_lookup(host).await.unwrap();

	if let Some(ip) = response.iter().next() {
		Ok(**ip)
	} else {
		Err(anyhow!("No A record found for {}", host))
	}
}

pub async fn resolve_target(host: &str, port: u16, srv_prefix: &str) -> Result<SocketAddrV4> {
	let resolver = Resolver::builder_tokio().unwrap().build();

	if !srv_prefix.is_empty() {
		if let Ok(srv_lookup) = resolver.srv_lookup(format!("{srv_prefix}.{host}")).await {
			let srv_result = srv_lookup.iter().next().unwrap();

			if let Ok(ip) = resolve_a(resolver.clone(), &srv_result.target().to_string()).await {
				return Ok(SocketAddrV4::new(ip, srv_result.port()));
			}
		}
	}

	let ip = resolve_a(resolver.clone(), host).await?;
	Ok(SocketAddrV4::new(ip, port))
}

pub async fn parse_target(input: &str, method: AttackMethod) -> Result<SocketAddrV4> {
	let (default_port, srv_prefix) = (method_to_port(method), method_to_srv_prefix(method));

	// If user specified a port
	if let Some((host, port_str)) = input.rsplit_once(":") {
		let port = port_str.parse().map_err(|_| anyhow!("Invalid port"))?;

		if let Ok(ip) = host.parse::<Ipv4Addr>() {
			return Ok(SocketAddrV4::new(ip, port));
		}

		return resolve_target(host, port, &srv_prefix).await;
	}

	// No port given, try SRV first
	return resolve_target(input, default_port, &srv_prefix).await;
}

pub fn parse_hostname(input: &str) -> String {
	// Remove the port
	if let Some((host, _)) = input.rsplit_once(":") {
		return host.to_string();
	}
	input.to_string()
}
