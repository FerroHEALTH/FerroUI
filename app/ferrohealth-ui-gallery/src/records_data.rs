// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The synthetic people, records and identifiers the patient-record
//! components draw (`pages::records`), as data.
//!
//! Every one is invented. The identifiers come from several countries'
//! systems side by side, and each is invalid for its scheme where the
//! scheme has a check, which the tests at the foot of this file prove: a
//! BSN-like number fails the eleven-proof, an NHS-like number its modulus
//! 11 check, a Belgian-like national number its modulus 97 check, a
//! NIR-like value carries a wrong key, an SSN-like value an area never
//! issued, and a personnummer-like value fails the Luhn check. The other
//! one is a local hospital number, which has no check.

/// One synthetic identifier: its key, its domain's words, its system and
/// its whole value, which the gallery's pretend server alone reads.
#[derive(Debug)]
pub struct Synthetic {
    /// The key a row and a reveal name it by.
    pub key: &'static str,
    /// The domain's short name.
    pub domain: &'static str,
    /// The system's URI, an example one.
    pub system: &'static str,
    /// The whole value: invented and invalid for its scheme.
    pub value: &'static str,
}

/// The gallery's identifiers, one per system, from several countries.
pub const IDENTIFIERS: &[Synthetic] = &[
    Synthetic {
        key: "bsn",
        domain: "BSN",
        system: "urn:example:nl:bsn",
        value: "999 000 124",
    },
    Synthetic {
        key: "nhs",
        domain: "NHS number",
        system: "urn:example:uk:nhs-number",
        value: "999 000 0011",
    },
    Synthetic {
        key: "nir",
        domain: "NIR",
        system: "urn:example:fr:nir",
        value: "1 99 01 99 000 000 01",
    },
    Synthetic {
        key: "rrn",
        domain: "Rijksregisternummer",
        system: "urn:example:be:rrn",
        value: "00.01.01-000.00",
    },
    Synthetic {
        key: "pnr",
        domain: "Personnummer",
        system: "urn:example:se:personnummer",
        value: "000101-0002",
    },
    Synthetic {
        key: "ssn",
        domain: "SSN",
        system: "urn:example:us:ssn",
        value: "900-00-0001",
    },
    Synthetic {
        key: "mrn",
        domain: "MRN · Synthetic General",
        system: "urn:example:hospital:mrn",
        value: "SYN-0000-0042",
    },
];

/// A source record of the compare: its key, title, quiet line, values of
/// the family name, given names, birth date and postal code, and the keys
/// of its identifiers.
#[derive(Debug)]
pub struct SourceRecord {
    /// The record's key.
    pub key: &'static str,
    /// The record's title.
    pub title: &'static str,
    /// The quiet line under the title.
    pub sub: &'static str,
    /// The values of the four fields, `None` where it has none.
    pub values: [Option<&'static str>; 4],
    /// The keys of its identifiers, in its order.
    pub ids: &'static [&'static str],
}

/// The leading record of the compare.
pub const LEADING: SourceRecord = SourceRecord {
    key: "lab",
    title: "Synthetic Laboratory",
    sub: "MRN · since 2019",
    values: [
        Some("Sample"),
        Some("Jordan Alex"),
        Some("1980-01-01"),
        Some("1000 AA"),
    ],
    ids: &["mrn", "bsn"],
};

/// The other records of the compare.
pub const OTHERS: [SourceRecord; 3] = [
    SourceRecord {
        key: "gp",
        title: "Synthetic Practice",
        sub: "NHS number · since 2021",
        values: [
            Some("Sample"),
            Some("Jordan Alex"),
            Some("1980-01-01"),
            Some("SW1A 0XX"),
        ],
        ids: &["nhs"],
    },
    SourceRecord {
        key: "clinic",
        title: "Clinique Synthétique",
        sub: "NIR · since 2023",
        values: [
            Some("Samplé"),
            Some("Jordan Alex"),
            Some("1980-01-01"),
            Some("1000 AA"),
        ],
        ids: &["nir", "rrn"],
    },
    SourceRecord {
        key: "registry",
        title: "Synthetic Registry",
        sub: "Personnummer · since 2024",
        values: [Some("Sample"), None, Some("1980-01-01"), None],
        ids: &["pnr"],
    },
];

/// The two records of the review queue's pair `number`.
#[must_use]
pub fn pair_names(number: u32) -> (String, String) {
    (
        format!("Sample {number:02}, Jordan"),
        format!("Sample {number:02}, Jordana"),
    )
}

/// The wait of the review queue's pair `number`, in days.
#[must_use]
pub fn pair_wait(number: u32) -> String {
    format!("{} d", 21_u32.saturating_sub(number))
}

/// A version's change of the family name: before and after.
pub const FAMILY_CHANGE: (&str, &str) = ("Sampel", "Sample");

/// A version's added postal code.
pub const POSTAL_ADDED: &str = "1000 AA";

#[cfg(test)]
mod tests {
    use super::IDENTIFIERS;

    /// The digits of `value`.
    fn digits(value: &str) -> Vec<u32> {
        value.chars().filter_map(|c| c.to_digit(10)).collect()
    }

    /// The value of the synthetic identifier `key`.
    fn value(key: &str) -> &'static str {
        IDENTIFIERS
            .iter()
            .find(|each| each.key == key)
            .map(|each| each.value)
            .expect("the gallery shows this identifier")
    }

    /// Whether `value` passes the Dutch BSN eleven-proof.
    fn eleven_proof(value: &str) -> bool {
        let d = digits(value);
        if d.len() != 9 {
            return false;
        }
        let weighted: u32 = d
            .iter()
            .take(8)
            .zip((2..=9).rev())
            .map(|(x, w)| x * w)
            .sum();
        (i64::from(weighted) - i64::from(d[8])).rem_euclid(11) == 0
    }

    /// Whether `value` passes the NHS number's modulus 11 check.
    fn nhs_check(value: &str) -> bool {
        let d = digits(value);
        if d.len() != 10 {
            return false;
        }
        let weighted: u32 = d
            .iter()
            .take(9)
            .zip((2..=10).rev())
            .map(|(x, w)| x * w)
            .sum();
        match 11 - weighted % 11 {
            11 => d[9] == 0,
            10 => false,
            check => d[9] == check,
        }
    }

    /// Whether `value` passes the Belgian national number's modulus 97
    /// check, for a birth before 2000 or from 2000.
    fn belgian_check(value: &str) -> bool {
        let d = digits(value);
        if d.len() != 11 {
            return false;
        }
        let base = d[..9]
            .iter()
            .fold(0_u64, |n, digit| n * 10 + u64::from(*digit));
        let check = u64::from(d[9] * 10 + d[10]);
        97 - base % 97 == check || 97 - (2_000_000_000 + base) % 97 == check
    }

    /// Whether `value` carries the right NIR key: 97 less the first
    /// thirteen digits modulo 97.
    fn nir_key(value: &str) -> bool {
        let d = digits(value);
        if d.len() != 15 {
            return false;
        }
        let base = d[..13]
            .iter()
            .fold(0_u64, |n, digit| n * 10 + u64::from(*digit));
        97 - base % 97 == u64::from(d[13] * 10 + d[14])
    }

    /// Whether `value` names an SSN area the Social Security Administration
    /// issues: never 000, 666 or 900 to 999.
    fn ssn_area_issued(value: &str) -> bool {
        let d = digits(value);
        let area = d.iter().take(3).fold(0_u32, |n, digit| n * 10 + digit);
        d.len() == 9 && area != 0 && area != 666 && area < 900
    }

    /// Whether `value` passes the Luhn check a Swedish personnummer carries
    /// over its last ten digits.
    fn luhn(value: &str) -> bool {
        let d = digits(value);
        let ten = &d[d.len().saturating_sub(10)..];
        let sum: u32 = ten
            .iter()
            .enumerate()
            .map(|(i, digit)| {
                if i % 2 == 0 {
                    let twice = digit * 2;
                    if twice > 9 { twice - 9 } else { twice }
                } else {
                    *digit
                }
            })
            .sum();
        ten.len() == 10 && sum.is_multiple_of(10)
    }

    /// Whether `check` accepts `value` with its last `width` digits made
    /// right: the example differs from a passing number only in its check,
    /// so the check is read as the scheme reads it. The passing number
    /// exists only while the test runs.
    fn passes_with_another_check(value: &str, width: usize, check: fn(&str) -> bool) -> bool {
        let all: String = value.chars().filter(char::is_ascii_digit).collect();
        let base = all.get(..all.len() - width).expect("the digits are ASCII");
        let tails = 10_u32.pow(u32::try_from(width).unwrap());
        (0..tails).any(|tail| check(&format!("{base}{tail:0width$}")))
    }

    #[test]
    fn the_bsn_like_number_fails_the_eleven_proof() {
        assert!(!eleven_proof(value("bsn")));
        assert!(passes_with_another_check(value("bsn"), 1, eleven_proof));
    }

    #[test]
    fn the_nhs_like_number_fails_its_modulus_11_check() {
        assert!(!nhs_check(value("nhs")));
        assert!(passes_with_another_check(value("nhs"), 1, nhs_check));
    }

    #[test]
    fn the_belgian_like_number_fails_its_modulus_97_check() {
        assert!(!belgian_check(value("rrn")));
        assert!(passes_with_another_check(value("rrn"), 2, belgian_check));
    }

    #[test]
    fn the_nir_like_value_carries_a_wrong_key() {
        assert!(!nir_key(value("nir")));
        assert!(passes_with_another_check(value("nir"), 2, nir_key));
    }

    #[test]
    fn the_ssn_like_value_uses_an_area_never_issued() {
        assert!(!ssn_area_issued(value("ssn")));
        let issued = value("ssn").replacen('9', "1", 1);
        assert!(ssn_area_issued(&issued), "the check reads an issued area");
    }

    #[test]
    fn the_personnummer_like_value_fails_the_luhn_check() {
        assert!(!luhn(value("pnr")));
        assert!(passes_with_another_check(value("pnr"), 1, luhn));
    }

    #[test]
    fn every_nine_digit_run_in_the_gallery_fails_the_eleven_proof() {
        for each in IDENTIFIERS {
            if digits(each.value).len() == 9 {
                assert!(!eleven_proof(each.value), "{} could be a BSN", each.key);
            }
        }
    }
}
