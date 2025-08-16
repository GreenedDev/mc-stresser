const UNITS: [(&str, u64); 3] = [
    ("seconds", 1),
    ("minutes", 60),
    ("hours", 3600),
];

fn extract_digits(s: &str) -> u64 {
    let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<u64>().expect("Duration doesn't contain any numbers")
}

fn extract_letters(s: &str) -> String {
    return s.chars().filter(|c| c.is_ascii_alphabetic()).collect();
}

pub fn parse_duration(input: &str) -> Result<(u64, String), String> {
    let input = input.trim().to_lowercase();
    let number = extract_digits(&input);
    
    let unit_str = match extract_letters(&input).as_str() {
    "" => "s".to_string(),
    s => s.to_string(),
    };

    if number == 0 {
        return Err("Duration can't be 0".to_string());
    }

    let mut matched: Option<(&str, u64)> = None;
    for (name, factor) in UNITS {
        if name.starts_with(&unit_str) {
            matched = Some((name, factor));
            break;
        }
    }

    let (canonical_name, factor) = matched.ok_or("Invalid unit, please use seconds, minutes or hours")?;
    let total_seconds = number * factor;

    let display_unit = if number == 1 {
        canonical_name.trim_end_matches("s")
    } else {
        canonical_name
    };

    Ok((total_seconds, format!("{number} {display_unit}")))
}
