use std::process::exit;

pub enum AttackMethod {
    Join,
    Ping,
}
pub fn parse_method(input: &str) -> AttackMethod {
    let input = input.to_lowercase();
    match input.as_str() {
        "join" => AttackMethod::Join,
        "ping" => AttackMethod::Ping,
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
    }
}
