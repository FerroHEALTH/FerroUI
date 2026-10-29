// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The query of an address, as the table model reads and writes it: the
//! pairs in the order sent, form-encoded (WHATWG URL, the
//! `application/x-www-form-urlencoded` serializer).

use std::fmt;

/// The pairs of a query, in order; a name may occur more than once.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Query(Vec<(String, String)>);

impl Query {
    /// An empty query.
    #[must_use]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// The query of `text`, the part of an address after its `?`, which may
    /// be empty.
    #[must_use]
    pub fn parse(text: &str) -> Self {
        Self(
            form_urlencoded::parse(text.as_bytes())
                .into_owned()
                .collect(),
        )
    }

    /// The first value of `name`, if any.
    #[must_use]
    pub fn get_str(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(each, _)| each == name)
            .map(|(_, value)| value.as_str())
    }

    /// Adds `value` under `name`, after any value it already has.
    pub fn insert(&mut self, name: &str, value: String) {
        self.0.push((name.to_owned(), value));
    }

    /// Sets `name` to `value` alone, removing every value it had.
    pub fn replace(&mut self, name: &str, value: String) {
        self.remove(name);
        self.insert(name, value);
    }

    /// Removes every value of `name`; `true` when there was one.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.0.len();
        self.0.retain(|(each, _)| each != name);
        self.0.len() != before
    }

    /// The pairs, in order.
    #[must_use]
    pub fn pairs(&self) -> &[(String, String)] {
        &self.0
    }

    /// The query as an address carries it: `?a=b&c=d`, or nothing for an
    /// empty query.
    #[must_use]
    pub fn to_query_string(&self) -> String {
        if self.0.is_empty() {
            return String::new();
        }
        let mut out = String::from("?");
        let mut encoder = form_urlencoded::Serializer::new(String::new());
        for (name, value) in &self.0 {
            encoder.append_pair(name, value);
        }
        out.push_str(&encoder.finish());
        out
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_query_string())
    }
}

impl From<Vec<(String, String)>> for Query {
    fn from(pairs: Vec<(String, String)>) -> Self {
        Self(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn a_query_reads_and_writes_its_pairs_in_order() {
        let mut query = Query::parse("q=a+b%26c&size=50");
        assert_eq!(Some("a b&c"), query.get_str("q"));
        query.replace("size", String::from("25"));
        query.insert("page", String::from("2"));
        assert_eq!("?q=a+b%26c&size=25&page=2", query.to_query_string());
        assert!(query.remove("page"));
        assert!(!query.remove("page"));
        assert_eq!("", Query::new().to_query_string());
    }
}
