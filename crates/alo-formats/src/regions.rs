//! **Every region a person may choose to write dates and times like**: the
//! 249 codes ISO 3166-1 assigns, and Kosovo.
//!
//! A list of codes rather than of names, and a standard's list rather than a
//! choice of ours. Which places are on it is the ISO 3166 Maintenance Agency's
//! answer, and the same on every machine in the world; what each is called is
//! CLDR's, in the person's language, and is asked in `crate::region_names`.
//!
//! **Not Europe's 27, and not the regions the 24 languages are spoken in.** A
//! Portuguese speaker in Belgium is this crate's own first example, and a
//! person in Brazil, Canada or Japan reading one of the 24 is no less a person
//! this machine is for — `CLAUDE.md`: *built in Europe, not only for Europe*.
//!
//! **What is left off, and why**: ISO's user-assigned codes (`AA`, `QM`–`QZ`,
//! `XA`–`XZ`, `ZZ`) name nothing, and its exceptionally reserved ones (`EU`,
//! `UN`, `UK` and the rest) are not places a calendar is kept in. CLDR knows
//! some of them; none of them is a way of writing a date.
//!
//! **The one exception is `XK`, Kosovo**, which ISO has not assigned and the
//! European Union, CLDR and most of the world use. Leaving a place where people
//! keep calendars off a European machine because a code is provisional would be
//! this list taking a side it has no business taking.

/// The officially assigned ISO 3166-1 alpha-2 codes and `XK`, in the order
/// ISO sorts them.
pub const EVERY_REGION: [&str; 250] = [
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS",
    "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", "DO", "DZ", "EC", "EE",
    "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE", "GF",
    "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK", "HM",
    "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE", "JM",
    "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC",
    "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK",
    "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA",
    "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG",
    "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW",
    "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS",
    "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO",
    "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
    "VN", "VU", "WF", "WS", "XK", "YE", "YT", "ZA", "ZM", "ZW",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each code is two capital letters, once, in order** — which is what
    /// `Regionally::in_region` will take, and what makes this list checkable.
    #[test]
    fn every_region_is_a_code_once_and_in_order() {
        for pair in EVERY_REGION.windows(2) {
            if let [one, other] = pair {
                assert!(one < other, "{one} before {other}");
            }
        }
        for code in EVERY_REGION {
            assert!(
                code.len() == 2 && code.chars().all(|letter| letter.is_ascii_uppercase()),
                "{code}"
            );
        }
    }

    /// **Nothing that names no place is offered.**
    #[test]
    fn no_code_that_names_no_place_is_offered() {
        for code in ["AA", "EU", "UN", "UK", "ZZ", "QO", "XA"] {
            assert!(!EVERY_REGION.contains(&code), "{code}");
        }
    }
}
