use crate::ast::{Expression, Literal};
use crate::parser::{parse_expression, Rule};
use pest::iterators::Pair;
use std::collections::HashMap;

pub(crate) fn parse_array_literal(pair: Pair<Rule>) -> Result<Expression, String> {
    let mut elements = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            elements.push(parse_expression(inner)?);
        }
    }

    Ok(Expression::Literal(Literal::Array(elements)))
}

pub(crate) fn parse_object_literal(pair: Pair<Rule>) -> Result<Expression, String> {
    let properties = HashMap::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {}
            Rule::pairs_literal => {
                return parse_pairs_literal(inner);
            }
            _ => {}
        }
    }

    Ok(Expression::Literal(Literal::Object(properties)))
}

pub(crate) fn parse_pairs_literal(pair: Pair<Rule>) -> Result<Expression, String> {
    let mut properties = HashMap::new();

    for pair_item in pair.into_inner() {
        if pair_item.as_rule() == Rule::pairs_item {
            let mut key = String::new();
            let mut value = None;

            for item in pair_item.into_inner() {
                match item.as_rule() {
                    Rule::string => {
                        let s = item.as_str();
                        key = s[1..s.len() - 1].to_string();
                    }
                    Rule::identifier => {
                        key = item.as_str().to_string();
                    }
                    Rule::expression => value = Some(parse_expression(item)?),
                    _ => {}
                }
            }

            if let Some(v) = value {
                properties.insert(key, v);
            }
        }
    }

    Ok(Expression::Literal(Literal::Object(properties)))
}
