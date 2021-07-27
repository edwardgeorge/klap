use pest::Parser;
use pest_derive::*;

use crate::types::*;

#[derive(Parser)]
#[grammar = "labels.pest"]
struct LabelParser;

fn match_key(part: pest::iterators::Pair<'_, Rule>) -> Key {
    match part.as_rule() {
        Rule::label_key => {
            let mut pairs = part.into_inner();
            let first = pairs.next().unwrap();
            let key = match first.as_rule() {
                Rule::dns_subdomain => {
                    let second = pairs.next().unwrap();
                    match second.as_rule() {
                        Rule::label_key_name => Key::new_with_prefix(
                            KeyPrefix(first.as_str().to_string()),
                            KeyName(second.as_str().to_string()),
                        ),
                        _ => unreachable!(),
                    }
                }
                Rule::label_key_name => Key::new_no_prefix(KeyName(first.as_str().to_string())),
                _ => unreachable!(),
            };
            assert!(pairs.next().is_none());
            key
        }
        _ => panic!("called with non-key rule"),
    }
}

fn match_label(part: pest::iterators::Pair<'_, Rule>) -> Label {
    let mut i = part.into_inner();
    let key = match_key(i.next().unwrap());
    let value = i.next().map(|v| v.as_str()).unwrap_or("");
    assert!(i.next().is_none());
    Label::new(key, LabelValue(value.to_string()))
}

pub fn label_keyprefix_from_str(input: &str) -> Result<KeyPrefix, Error> {
    let mut pairs = LabelParser::parse(Rule::label_keyprefix_whole, input)?;
    let first = pairs.next().unwrap();
    let prefix = match first.as_rule() {
        Rule::dns_subdomain => KeyPrefix(first.as_str().to_string()),
        _ => unreachable!(),
    };
    match pairs.next().unwrap().as_rule() {
        Rule::EOI => Ok(prefix),
        _ => unreachable!(),
    }
}

pub fn label_keyname_from_str(input: &str) -> Result<KeyName, Error> {
    let mut pairs = LabelParser::parse(Rule::label_keyname_whole, input)?;
    let first = pairs.next().unwrap();
    let name = match first.as_rule() {
        Rule::label_key_name => KeyName(first.as_str().to_string()),
        _ => unreachable!(),
    };
    match pairs.next().unwrap().as_rule() {
        Rule::EOI => Ok(name),
        _ => unreachable!(),
    }
}

pub fn label_key_from_str(input: &str) -> Result<Key, Error> {
    let mut pairs = LabelParser::parse(Rule::label_key_whole, input)?;
    let first = pairs.next().unwrap();
    let key = match first.as_rule() {
        Rule::label_key => match_key(first),
        _ => unreachable!(),
    };
    match pairs.next().unwrap().as_rule() {
        Rule::EOI => Ok(key),
        _ => unreachable!(),
    }
}

pub fn label_value_from_str(input: &str) -> Result<LabelValue, Error> {
    let mut pairs = LabelParser::parse(Rule::label_value_whole, input)?;
    let first = pairs.next().unwrap();
    let value = match first.as_rule() {
        Rule::label_value => LabelValue(first.as_str().to_string()),
        Rule::EOI => return Ok(LabelValue("".to_string())),
        _ => unreachable!(),
    };
    match pairs.next().unwrap().as_rule() {
        Rule::EOI => Ok(value),
        _ => unreachable!(),
    }
}

pub fn label_from_envstr(input: &str) -> Result<Label, Error> {
    let mut pairs = LabelParser::parse(Rule::label_whole, input)?;
    let first = pairs.next().unwrap();
    let label = match first.as_rule() {
        Rule::label => match_label(first),
        _ => unreachable!(),
    };
    match pairs.next().unwrap().as_rule() {
        Rule::EOI => Ok(label),
        _ => unreachable!(),
    }
}

pub fn labels_from_envstr(input: &str) -> Result<Vec<Label>, Error> {
    let mut res = Vec::new();
    for pair in LabelParser::parse(Rule::labels, input)? {
        match pair.as_rule() {
            Rule::label => res.push(match_label(pair)),
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok(res)
}

pub fn label_from_str_wcolon(input: &str) -> Result<Label, Error> {
    let mut pairs = LabelParser::parse(Rule::label_colon_whole, input)?;
    let first = pairs.next().unwrap();
    let label = match first.as_rule() {
        Rule::label_colon_spec => match_label(first),
        _ => unreachable!(),
    };
    match pairs.next().unwrap().as_rule() {
        Rule::EOI => Ok(label),
        _ => unreachable!(),
    }
}

pub fn labels_from_csvstr_wcolon(input: &str) -> Result<Vec<Label>, Error> {
    let mut res = Vec::new();
    for pair in LabelParser::parse(Rule::labels_colon_csv, input)? {
        match pair.as_rule() {
            Rule::label_colon_spec => res.push(match_label(pair)),
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok(res)
}

pub fn labels_from_wsvstr_wcolon(input: &str) -> Result<Vec<Label>, Error> {
    let mut res = Vec::new();
    for pair in LabelParser::parse(Rule::labels_colon_wsv, input)? {
        match pair.as_rule() {
            Rule::label_colon_spec => res.push(match_label(pair)),
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok(res)
}

pub fn labels_from_str_either(input: &str) -> Result<Vec<Label>, Error> {
    let mut res = Vec::new();
    for pair in LabelParser::parse(Rule::labels_colon_either, input)? {
        match pair.as_rule() {
            Rule::label_colon_spec => res.push(match_label(pair)),
            Rule::EOI => (),
            _ => unreachable!(),
        }
    }
    Ok(res)
}
