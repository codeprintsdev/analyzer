use codeprints_analyzer::Parser;
use std::fs;

fn ignore_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn test_parse_torvalds() {
    let raw = fs::read_to_string("fixtures/torvalds-2019-git.txt").unwrap();
    let mut parser = Parser::new(raw);
    let timeline = parser.parse().unwrap();
    let actual = serde_json::to_string_pretty(&timeline).unwrap();
    let expected = fs::read_to_string("fixtures/torvalds-2019.json").unwrap();
    assert_eq!(ignore_whitespace(&actual), ignore_whitespace(&expected));
}
