use std::process::exit;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AttackMethod {
    Join,
    Ping,
    Icmp,
}

pub fn parse_method(input: &str) -> AttackMethod {
    let input = input.to_lowercase();
    
    match input.as_str() {
        "join" => AttackMethod::Join,
        "ping" => AttackMethod::Ping,
        "icmp" => AttackMethod::Icmp,
        _ => {
            println!("Attack method {input} doesn't exist!");
            exit(0);
        }
    }
}

pub fn method_to_string(method: AttackMethod) -> String {
    match method {
        AttackMethod::Join => String::from("Join"),
        AttackMethod::Ping => String::from("Ping"),
        AttackMethod::Icmp => String::from("ICMP"),
    }
}

pub fn method_to_port(method: AttackMethod) -> u16 {
    match method {
        AttackMethod::Join => 25565,
        AttackMethod::Ping => 25565,
        AttackMethod::Icmp => 0,
    }
}

pub fn method_to_srv_prefix(method: AttackMethod) -> String {
    match method {
        AttackMethod::Join => String::from("_minecraft._tcp"),
        AttackMethod::Ping => String::from("_minecraft._tcp"),
        AttackMethod::Icmp => String::from(""),
    }
}
