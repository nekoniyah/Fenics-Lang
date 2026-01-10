use crate::ast::{Expression, StringPart};
use crate::parser::{parse_expression, FenicsParser, Rule};
use pest::iterators::Pair;
use pest::Parser;

/// Parse a string interpolation literal into an `Expression::StringInterpolation` node.
pub(crate) fn parse_string_interpolation(pair: Pair<Rule>) -> Result<Expression, String> {
    let s = pair.as_str();
    let content = &s[1..s.len() - 1]; // Strip surrounding quotes

    let mut parts = Vec::new();
    let mut current_text = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '#' {
            if let Some(&'{') = chars.peek() {
                chars.next(); // consume '{'

                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                let mut expr_str = String::new();
                let mut depth = 1;
                while let Some(ch) = chars.next() {
                    match ch {
                        '{' => {
                            depth += 1;
                            expr_str.push(ch);
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            expr_str.push(ch);
                        }
                        _ => expr_str.push(ch),
                    }
                }

                let expr_pairs = FenicsParser::parse(Rule::expression, &expr_str)
                    .map_err(|e| format!("Error parsing interpolation expression: {}", e))?;
                let expr_pair = expr_pairs
                    .into_iter()
                    .next()
                    .ok_or("No expression found in interpolation")?;
                parts.push(StringPart::Expression(Box::new(parse_expression(
                    expr_pair,
                )?)));
            } else {
                current_text.push(ch);
            }
        } else {
            current_text.push(ch);
        }
    }

    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }

    Ok(Expression::StringInterpolation { parts })
}

/// Evaluate string parts using the provided expression evaluator.
pub(crate) fn evaluate_string_parts<E>(
    parts: &[StringPart],
    mut eval_expr: E,
) -> Result<String, String>
where
    E: FnMut(&Expression) -> Result<String, String>,
{
    let mut result = String::new();

    for part in parts {
        match part {
            StringPart::Text(text) => result.push_str(text),
            StringPart::Expression(expr) => result.push_str(&eval_expr(expr)?),
        }
    }

    Ok(result)
}
