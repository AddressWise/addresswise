pub fn normalize_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut last_was_space = false;

    for c in input.to_lowercase().chars() {
        if let Some(folded) = fold_alphanumeric(c) {
            normalized.push(folded);
            last_was_space = false;
        } else if c.is_whitespace() || matches!(c, ',' | '.' | '-' | '/' | '#' | ':' | ';') {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        }
    }

    normalized.trim().to_string()
}

pub fn compact_alphanumeric(input: &str) -> String {
    normalize_text(input)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

pub fn canonical_country_code(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = normalize_text(trimmed);
    
    // 1. Check if it's already a 2-letter ISO code
    if normalized.len() == 2 {
        let upper = normalized.to_ascii_uppercase();
        if upper.chars().all(|c| c.is_ascii_alphabetic()) && is_iso_country_code(&upper) {
            return Some(upper);
        }
    }

    // 2. Map common country names
    match normalized.as_str() {
        "germany" | "deutschland" | "de" => Some("DE".to_string()),
        "france" | "fr" => Some("FR".to_string()),
        "united kingdom" | "uk" | "gb" | "great britain" => Some("GB".to_string()),
        "united states" | "usa" | "us" => Some("US".to_string()),
        "italy" | "italia" | "it" => Some("IT".to_string()),
        "spain" | "espana" | "es" => Some("ES".to_string()),
        "czech republic" | "czechia" | "cesko" | "ceska republika" | "cz" => Some("CZ".to_string()),
        "poland" | "polska" | "pl" => Some("PL".to_string()),
        "slovakia" | "slovensko" | "sk" => Some("SK".to_string()),
        "austria" | "osterreich" | "at" => Some("AT".to_string()),
        "hungary" | "magyarorszag" | "hu" => Some("HU".to_string()),
        "netherlands" | "nederland" | "nl" => Some("NL".to_string()),
        "belgium" | "belgie" | "belgique" | "be" => Some("BE".to_string()),
        "switzerland" | "schweiz" | "suisse" | "svizzera" | "ch" => Some("CH".to_string()),
        _ => None,
    }
}

fn is_iso_country_code(code: &str) -> bool {
    const ISO_ALPHA2: &[&str] = &[
        "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX",
        "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ",
        "BR", "BS", "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK",
        "CL", "CM", "CN", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM",
        "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR",
        "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS",
        "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN",
        "IO", "IQ", "IR", "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN",
        "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV",
        "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK", "ML", "MM", "MN", "MO", "MP", "MQ",
        "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI",
        "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM",
        "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW", "SA", "SB", "SC",
        "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS", "ST", "SV",
        "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR",
        "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
        "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
    ];

    ISO_ALPHA2.contains(&code)
}

fn fold_alphanumeric(c: char) -> Option<char> {
    let folded = match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ą' => 'a',
        'æ' => 'a',
        'ç' | 'č' | 'ć' => 'c',
        'ď' => 'd',
        'é' | 'ě' | 'è' | 'ê' | 'ë' | 'ę' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ľ' | 'ĺ' | 'ł' => 'l',
        'ñ' | 'ň' => 'n',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' => 'o',
        'ř' => 'r',
        'š' | 'ś' => 's',
        'ť' => 't',
        'ú' | 'ů' | 'ù' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        'ž' | 'ź' | 'ż' => 'z',
        _ if c.is_ascii_alphanumeric() => c,
        _ => return None,
    };

    Some(folded)
}

#[cfg(test)]
mod tests {
    use super::{canonical_country_code, compact_alphanumeric, normalize_text};

    #[test]
    fn normalizes_diacritics_and_punctuation() {
        assert_eq!(
            normalize_text("Avenue de France 123, Stiring-Wendel"),
            "avenue de france 123 stiring wendel"
        );
    }

    #[test]
    fn compacts_postal_codes() {
        assert_eq!(compact_alphanumeric("NW1 6XE"), "nw16xe");
    }

    #[test]
    fn accepts_two_letter_country_codes() {
        assert_eq!(canonical_country_code(" fr "), Some("FR".to_string()));
        assert_eq!(canonical_country_code("ny"), None);
        assert_eq!(canonical_country_code("France"), Some("FR".to_string()));
        assert_eq!(canonical_country_code("Cesko"), Some("CZ".to_string()));
    }
}
