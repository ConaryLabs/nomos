//! The canonical byte profile of `KERNEL.md` section 7.

use nomos_core::canonical::keyed_array;
use nomos_core::canonical::read::{is_canonical, parse_canonical};
use nomos_core::id::{EntityId, StableId};
use nomos_core::{CanonicalValue, FieldName};

fn field(name: &str) -> FieldName {
    FieldName::new(name).unwrap()
}

#[test]
fn encoding_is_byte_stable_across_insertion_order() {
    let pairs = [
        (field("ward"), CanonicalValue::text("sealed")),
        (field("access"), CanonicalValue::text("locked")),
        (field("integrity"), CanonicalValue::text("intact")),
        (field("combustion"), CanonicalValue::text("cold")),
    ];

    let forward = CanonicalValue::object(pairs.clone()).unwrap();
    let reversed = CanonicalValue::object(pairs.iter().rev().cloned()).unwrap();
    let shuffled = CanonicalValue::object([
        pairs[2].clone(),
        pairs[0].clone(),
        pairs[3].clone(),
        pairs[1].clone(),
    ])
    .unwrap();

    let expected =
        br#"{"access":"locked","combustion":"cold","integrity":"intact","ward":"sealed"}"#.to_vec();
    assert_eq!(forward.to_canonical_bytes(), expected);
    assert_eq!(reversed.to_canonical_bytes(), expected);
    assert_eq!(shuffled.to_canonical_bytes(), expected);
}

#[test]
fn arrays_ordered_by_stable_id_do_not_depend_on_declaration_order() {
    let entity = |name: &str| {
        let id = EntityId::parse(name).unwrap();
        (
            id.clone(),
            CanonicalValue::object([(field("id"), id.to_canonical())]).unwrap(),
        )
    };

    let declared = keyed_array([
        entity("north_gate"),
        entity("brazier_02"),
        entity("flooded_section"),
    ])
    .unwrap();
    let other_order = keyed_array([
        entity("flooded_section"),
        entity("north_gate"),
        entity("brazier_02"),
    ])
    .unwrap();

    assert_eq!(
        declared.to_canonical_bytes(),
        other_order.to_canonical_bytes()
    );
    assert_eq!(
        declared.to_canonical_bytes(),
        br#"[{"id":"brazier_02"},{"id":"flooded_section"},{"id":"north_gate"}]"#.to_vec()
    );
}

#[test]
fn dynamic_objects_and_keyed_arrays_reject_duplicate_identity() {
    let duplicate_field = CanonicalValue::object([
        (field("state"), CanonicalValue::text("locked")),
        (field("state"), CanonicalValue::text("open")),
    ])
    .unwrap_err();
    assert_eq!(duplicate_field.code().as_str(), "EK0304");

    let duplicate_id = EntityId::parse("north_gate").unwrap();
    let duplicate_key = keyed_array([
        (duplicate_id.clone(), CanonicalValue::text("first")),
        (duplicate_id, CanonicalValue::text("second")),
    ])
    .unwrap_err();
    assert_eq!(duplicate_key.code().as_str(), "EK0304");
}

#[test]
#[should_panic(expected = "duplicate declared canonical field `state`")]
fn duplicate_declared_fields_are_developer_errors() {
    let _ = CanonicalValue::object_declared([
        ("state", CanonicalValue::text("locked")),
        ("state", CanonicalValue::text("open")),
    ]);
}

#[test]
fn integers_are_the_only_numbers_and_have_one_spelling() {
    assert_eq!(CanonicalValue::Int(0).to_canonical_bytes(), b"0".to_vec());
    assert_eq!(CanonicalValue::Int(-3).to_canonical_bytes(), b"-3".to_vec());
    assert_eq!(
        CanonicalValue::Uint(u64::MAX).to_canonical_bytes(),
        b"18446744073709551615".to_vec()
    );

    // No float variant exists, so an authoritative artifact cannot carry one.
    // Reading one back is refused rather than rounded.
    let rejected = parse_canonical(br#"{"cost":3.0}"#).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0302");
    assert!(rejected.message().contains("floating-point"));
    assert!(parse_canonical(br#"{"cost":3e2}"#).is_err());
}

#[test]
fn strings_escape_only_what_the_profile_requires() {
    let value = CanonicalValue::text(
        "quote\" reverse\\ solidus/ backspace\u{8} formfeed\u{c} newline\n carriage\r \
         tab\t nul\u{0} bell\u{7} unit_separator\u{1f} caf\u{e9} del\u{7f}",
    );
    let bytes = value.to_canonical_bytes();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert_eq!(
        text,
        "\"quote\\\" reverse\\\\ solidus/ backspace\\b formfeed\\f newline\\n carriage\\r \
         tab\\t nul\\u0000 bell\\u0007 unit_separator\\u001f caf\u{e9} del\u{7f}\""
    );
    // Non-ASCII is emitted as UTF-8, never as an escape.
    assert!(bytes.windows(2).any(|pair| pair == [0xc3, 0xa9]));
    assert_eq!(parse_canonical(&bytes).unwrap(), value);
}

#[test]
fn the_reader_refuses_everything_the_profile_forbids() {
    let canonical = br#"{"a":1,"b":[true,null]}"#;
    assert!(is_canonical(canonical));

    let rejected: [(&[u8], &str, &str); 15] = [
        (b"{\"b\":1,\"a\":2}", "EK0303", "unsorted keys"),
        // The reader never skips whitespace, so this is refused structurally
        // rather than by the re-encode comparison.
        (b"{\"a\": 1}", "EK0302", "insignificant whitespace"),
        (b"{\"a\":01}", "EK0303", "redundant leading zero"),
        (b"{\"a\":+1}", "EK0302", "leading plus"),
        (
            b"{\"a\":\"\\u0041\"}",
            "EK0303",
            "escape where UTF-8 belongs",
        ),
        (b"{\"a\":\"\\/\"}", "EK0303", "escaped solidus"),
        (b"{\"a\":\"\\u0008\"}", "EK0303", "long backspace escape"),
        (b"{\"a\":\"\\u000c\"}", "EK0303", "long form-feed escape"),
        (b"{\"a\":\"\\u000a\"}", "EK0303", "long line-feed escape"),
        (
            b"{\"a\":\"\\u000d\"}",
            "EK0303",
            "long carriage-return escape",
        ),
        (b"{\"a\":\"\\u0009\"}", "EK0303", "long tab escape"),
        (b"{\"a\":\"\\u001F\"}", "EK0303", "uppercase hex escape"),
        (b"{\"a\":\"\\u007f\"}", "EK0303", "escaped delete character"),
        (b"{\"a\":1,\"a\":2}", "EK0303", "duplicate key"),
        (b"{\"a\":1}\n", "EK0302", "trailing newline"),
    ];
    for (bytes, code, why) in rejected {
        let diagnostic = parse_canonical(bytes).unwrap_err();
        assert_eq!(
            diagnostic.code().as_str(),
            code,
            "{why}: {}",
            String::from_utf8_lossy(bytes)
        );
    }

    // A byte-order mark is not whitespace and is not a value.
    assert!(parse_canonical("\u{feff}{}".as_bytes()).is_err());
    // Object keys use the same restricted shape as identifiers.
    assert_eq!(
        parse_canonical(br#"{"Alpha":1}"#)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0301"
    );
}

#[test]
fn canonical_field_names_use_the_exact_ascii_identifier_shape() {
    for accepted in ["a", "a0", "a_b", "schema_v2"] {
        assert_eq!(FieldName::new(accepted).unwrap().as_str(), accepted);
    }

    for rejected in ["", "0a", "_a", "Alpha", "a-b", "caf\u{e9}", "a\u{e9}"] {
        let diagnostic = FieldName::new(rejected).unwrap_err();
        assert_eq!(diagnostic.code().as_str(), "EK0301", "{rejected:?}");
    }
}

#[test]
fn every_canonical_encoding_reads_back_to_the_same_bytes() {
    let value = CanonicalValue::object([
        (field("empty_array"), CanonicalValue::Array(vec![])),
        (field("empty_object"), CanonicalValue::object([]).unwrap()),
        (field("flag"), CanonicalValue::Bool(false)),
        (field("nothing"), CanonicalValue::Null),
        (
            field("nested"),
            CanonicalValue::Array(vec![
                CanonicalValue::Int(-1),
                CanonicalValue::object([(field("deep"), CanonicalValue::text(""))]).unwrap(),
            ]),
        ),
    ])
    .unwrap();
    let bytes = value.to_canonical_bytes();
    let reread = parse_canonical(&bytes).unwrap();
    assert_eq!(reread.to_canonical_bytes(), bytes);
    assert!(
        !bytes.ends_with(b"\n"),
        "hashed bytes carry no trailing newline"
    );
}
