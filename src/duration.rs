fn extract_digits(s: &str) -> u64 {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits
        .parse::<u64>()
        .expect("Couldn't get number from duration")
}
pub fn parse_duration_as_secs(input: String) -> (u64, String) {
    let input = input.to_lowercase();
    for char in input.chars() {
        if char.is_numeric() || char.eq(&'s') || char.eq(&'m') || char.eq(&'h') {
            continue;
        }
        break;
    }
    let number = extract_digits(&input);
    let blank_or_s = match number {
        1 => "",
        _ => "s",
    };
    if input.contains("s") {
        (number, format!("{number} second{blank_or_s}"))
    } else if input.contains("m") {
        (number * 60, format!("{number} minute{blank_or_s}"))
    } else if input.contains("h") {
        (number * 3600, format!("{number} hour{blank_or_s}"))
    } else {
        (number, format!("{number} second{blank_or_s}"))
    }
}
