mod parser;
mod types;
pub use parser::*;
pub use types::*;

#[cfg(feature = "serde_support")]
#[cfg(test)]
mod serde_tests {
    use super::{Key, KeyName, KeyPrefix, LabelMap, LabelValue};
    use serde_yaml::{from_str, to_string};

    #[test]
    fn deser_hash() {
        let input = concat!(
            "foo: bar\n",
            "edwardgeorge.github.io/test-label: foo-bar\n",
            "empty-label: \"\"\n"
        );
        let parsed: LabelMap = from_str(input).unwrap();
        let expected: LabelMap = vec![
            (
                Key::new_no_prefix(KeyName("foo".to_string())),
                LabelValue("bar".to_string()),
            ),
            (
                Key::new_with_prefix(
                    KeyPrefix("edwardgeorge.github.io".to_string()),
                    KeyName("test-label".to_string()),
                ),
                LabelValue("foo-bar".to_string()),
            ),
            (
                Key::new_no_prefix(KeyName("empty-label".to_string())),
                LabelValue("".to_string()),
            ),
        ]
        .drain(..)
        .collect();
        assert_eq!(parsed, expected);
        // test the ser/de roundtrip
        assert_eq!(parsed, from_str(&to_string(&parsed).unwrap()).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("foo", None, "foo")]
    #[case("foo/bar", Some("foo"), "bar")]
    #[case("foo.bar-baz/qux", Some("foo.bar-baz"), "qux")]
    #[case(
        "edwardgeorge.github.io/example-label",
        Some("edwardgeorge.github.io"),
        "example-label"
    )]
    //#[case(&format!("{}{}/foo", "1234567890".repeat(6), "123"), None, "")] // over 255 chars
    fn test_can_parse_key(#[case] input: &str, #[case] prefix: Option<&str>, #[case] name: &str) {
        let key = label_key_from_str(input).unwrap();
        assert_eq!(key.to_string(), input.to_string());
        assert_eq!(key.prefix(), prefix);
        assert_eq!(key.name(), name);

        let key2 = Key::parse_str(input).unwrap();
        assert_eq!(key, key2);
    }

    #[test]
    fn test_long_subdomains() {
        let max_part: String = format!("{}{}", "1234567890".repeat(6), "123"); // 63 chars
        let long_subdomain: String =
            vec![&max_part[..], &max_part[..], &max_part[..], &max_part[..]].join("."); // just under 255 in total
        let prefix = label_keyprefix_from_str(&long_subdomain).unwrap();
        assert_eq!(prefix.as_str(), &long_subdomain);
        let prefix2 = KeyPrefix::parse_str(&long_subdomain).unwrap();
        assert_eq!(prefix, prefix2);

        let longer_subdomain: String = format!("{}.{}", long_subdomain, max_part);
        assert!(label_keyprefix_from_str(&longer_subdomain).is_err());
    }

    #[rstest]
    #[case("foo-")]
    #[case("foo-/bar")]
    #[case("foo./bar")]
    #[case("foo_bar/baz")]
    #[case(".foo./bar")]
    #[case("-foo/bar")]
    #[case("0123456789012345678901234567890134567890123456789012345678901234")] // over 63 chars
    #[case(&format!("{}{}/foo", "1234567890".repeat(6), "1234"))] // single label over 63 chars
    fn test_invalid_key(#[case] input: &str) {
        assert!(label_key_from_str(input).is_err());
    }

    #[rstest]
    #[case("foo-bar")]
    #[case(&format!("{}{}", "1234567890".repeat(6), "123"))]
    #[case("")]
    fn test_can_parse_value(#[case] input: &str) {
        let value = label_value_from_str(input).unwrap();
        let value2 = LabelValue::parse_str(input).unwrap();
        assert_eq!(input, value.as_str());
        assert_eq!(value, value2);
    }

    #[rstest]
    #[case("foo:bar", "foo", "bar")]
    #[case("foo/bar:baz", "foo/bar", "baz")]
    #[case("foo:", "foo", "")]
    fn test_parse_wcolon(#[case] input: &str, #[case] key: &str, #[case] value: &str) {
        let label = label_from_str_wcolon(input).unwrap();
        assert_eq!(&label.key.to_string(), key);
        assert_eq!(label.value.as_str(), value);
    }

    #[rstest]
    #[case("foo:bar", 1)]
    #[case("foo:bar,foo/bar:baz", 2)]
    fn test_labels_from_csv(#[case] input: &str, #[case] num_items: usize) {
        let labels = labels_from_csvstr_wcolon(input).unwrap();
        assert_eq!(labels.len(), num_items);
    }

    #[rstest]
    #[case("")]
    #[case("foo:bar,")]
    #[case("foo:bar,bar")]
    #[case("foo:bar,bar,baz:qux")]
    fn test_invalid_csv(#[case] input: &str) {
        assert!(labels_from_csvstr_wcolon(input).is_err());
    }

    #[rstest]
    #[case("foo:bar", 1)]
    #[case("foo:bar bar:baz", 2)]
    #[case("foo:bar\nbar:baz", 2)]
    #[case("foo:bar\n\n\n\nbar:baz", 2)]
    #[case("foo:bar     bar:baz", 2)]
    #[case("foo:bar \n    bar:baz", 2)]
    #[case("foo:bar bar:baz\nbaz: qux:qux", 4)]
    fn test_labels_from_wsv(#[case] input: &str, #[case] num_items: usize) {
        let labels = labels_from_wsvstr_wcolon(input).unwrap();
        assert_eq!(labels.len(), num_items);
    }

    #[rstest]
    #[case("foo:bar", 1)]
    #[case("foo:bar bar:baz", 2)]
    #[case("foo:bar,bar:baz", 2)]
    #[case("foo:bar bar:baz \nbaz:qux", 3)]
    #[case("foo:bar,bar:baz,baz:qux", 3)]
    fn test_labels_from_either(#[case] input: &str, #[case] num_items: usize) {
        let labels = labels_from_str_either(input).unwrap();
        assert_eq!(labels.len(), num_items);
    }

    #[rstest]
    #[case("")]
    #[case("foo:bar bar:baz,baz:qux")] // mixed separators
    #[case("foo:bar,bar:baz baz:qux")] // mixed separators
    fn test_invalid_labels_from_either(#[case] input: &str) {
        assert!(labels_from_str_either(input).is_err())
    }
}
