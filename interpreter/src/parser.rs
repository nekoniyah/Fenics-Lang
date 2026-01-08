use crate::ast::*;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "../grammar/fenics.pest"]
pub struct FenicsParser;

pub fn parse_program(input: &str) -> Result<Program, String> {
    let pairs =
        FenicsParser::parse(Rule::main, input).map_err(|e| format!("Parse error: {}", e))?;

    let mut statements = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::main => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::statement => {
                            if let Some(stmt) = parse_statement(inner_pair)? {
                                statements.push(stmt);
                            }
                        }
                        Rule::EOI => break,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Program { statements })
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> Result<Option<Statement>, String> {
    let inner = pair.into_inner().next();

    if inner.is_none() {
        return Ok(None);
    }

    let inner = inner.unwrap();

    match inner.as_rule() {
        Rule::const_definition => Ok(Some(parse_const_definition(inner)?)),
        Rule::mutable_definition => Ok(Some(parse_mutable_definition(inner)?)),
        Rule::global_const_definition => Ok(Some(parse_global_const_definition(inner)?)),
        Rule::global_mutable_definition => Ok(Some(parse_global_mutable_definition(inner)?)),
        Rule::assignment => Ok(Some(parse_assignment(inner)?)),
        Rule::increment_stmt => Ok(Some(parse_increment_stmt(inner)?)),
        Rule::function_def => Ok(Some(parse_function_def(inner)?)),
        Rule::if_stmt => Ok(Some(parse_if_stmt(inner)?)),
        Rule::for_loop => Ok(Some(parse_for_loop(inner)?)),
        Rule::while_loop => Ok(Some(parse_while_loop(inner)?)),
        Rule::loop_stmt => Ok(Some(parse_loop_stmt(inner)?)),
        Rule::try_catch => Ok(Some(parse_try_catch(inner)?)),
        Rule::return_stmt => Ok(Some(parse_return_stmt(inner)?)),
        Rule::lib_export => Ok(Some(parse_lib_export(inner)?)),
        Rule::import_stmt => Ok(Some(parse_import_stmt(inner)?)),
        Rule::expression => Ok(Some(Statement::Expression(parse_expression(inner)?))),
        _ => Ok(None),
    }
}

fn parse_lib_export(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut name = String::new();
    let mut exports: Vec<String> = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                if name.is_empty() {
                    name = inner.as_str().to_string();
                } else {
                    // Unexpected extra identifier
                }
            }
            Rule::lib_item => {
                for item in inner.into_inner() {
                    if item.as_rule() == Rule::identifier {
                        exports.push(item.as_str().to_string());
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Statement::LibExport { name, exports })
}

fn parse_import_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut path = None;
    let mut alias = None;

    let mut expect_alias = false;
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string => {
                let s = inner.as_str();
                path = Some(s[1..s.len() - 1].to_string());
            }
            Rule::identifier => {
                if !expect_alias && path.is_none() {
                    // This is the module name (pathless import)
                    path = Some(inner.as_str().to_string());
                } else if expect_alias {
                    alias = Some(inner.as_str().to_string());
                    expect_alias = false;
                }
            }
            Rule::as_keyword => {
                expect_alias = true;
            }
            _ => {}
        }
    }

    Ok(Statement::Import {
        path: path.ok_or("Missing path in import statement")?,
        alias,
    })
}

fn parse_const_definition(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut type_annotation = None;
    let mut name = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::r#type => type_annotation = Some(parse_type(inner)?),
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::expression => value = Some(parse_expression(inner)?),
            _ => {}
        }
    }

    Ok(Statement::VariableDeclaration {
        type_annotation,
        is_const: true,
        is_global: false,
        name,
        value: value.ok_or("Missing value in const definition")?,
    })
}

fn parse_mutable_definition(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut type_annotation = None;
    let mut name = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::r#type => type_annotation = Some(parse_type(inner)?),
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::expression => value = Some(parse_expression(inner)?),
            _ => {}
        }
    }

    Ok(Statement::VariableDeclaration {
        type_annotation,
        is_const: false,
        is_global: false,
        name,
        value: value.ok_or("Missing value in mutable definition")?,
    })
}

fn parse_global_const_definition(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut type_annotation = None;
    let mut name = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::r#type => type_annotation = Some(parse_type(inner)?),
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::expression => value = Some(parse_expression(inner)?),
            _ => {}
        }
    }

    Ok(Statement::VariableDeclaration {
        type_annotation,
        is_const: true,
        is_global: true,
        name,
        value: value.ok_or("Missing value in global const definition")?,
    })
}

fn parse_global_mutable_definition(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut type_annotation = None;
    let mut name = String::new();
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::r#type => type_annotation = Some(parse_type(inner)?),
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::expression => value = Some(parse_expression(inner)?),
            _ => {}
        }
    }

    Ok(Statement::VariableDeclaration {
        type_annotation,
        is_const: false,
        is_global: true,
        name,
        value: value.ok_or("Missing value in global mutable definition")?,
    })
}

fn parse_assignment(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut target = None;
    let mut op = BinaryOperator::Assign;
    let mut value = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                target = Some(Expression::Identifier(inner.as_str().to_string()));
            }
            Rule::dot_access => {
                target = Some(parse_dot_access(inner)?);
            }
            Rule::bracket_access => {
                target = Some(parse_bracket_access(inner)?);
            }
            Rule::assign => op = BinaryOperator::Assign,
            Rule::add_assign => op = BinaryOperator::AddAssign,
            Rule::sub_assign => op = BinaryOperator::SubAssign,
            Rule::mul_assign => op = BinaryOperator::MulAssign,
            Rule::div_assign => op = BinaryOperator::DivAssign,
            Rule::mod_assign => op = BinaryOperator::ModAssign,
            Rule::expression => value = Some(parse_expression(inner)?),
            _ => {}
        }
    }

    // Convert assignment to a statement
    // For now, we'll represent it as an expression statement with a binary op
    let target_expr = target.ok_or("Missing target in assignment")?;
    let value_expr = value.ok_or("Missing value in assignment")?;

    Ok(Statement::Expression(Expression::BinaryOp {
        left: Box::new(target_expr),
        op,
        right: Box::new(value_expr),
    }))
}

fn parse_increment_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut target = None;
    let mut is_increment = true;

    for inner in pair.into_inner() {
        match inner.as_str() {
            "++" => is_increment = true,
            "--" => is_increment = false,
            _ => match inner.as_rule() {
                Rule::identifier => {
                    target = Some(Expression::Identifier(inner.as_str().to_string()));
                }
                Rule::dot_access => {
                    target = Some(parse_dot_access(inner)?);
                }
                Rule::bracket_access => {
                    target = Some(parse_bracket_access(inner)?);
                }
                _ => {}
            },
        }
    }

    let target_expr = target.ok_or("Missing target in increment/decrement")?;
    let op = if is_increment {
        UnaryOperator::Increment
    } else {
        UnaryOperator::Decrement
    };

    Ok(Statement::Expression(Expression::UnaryOp {
        op,
        operand: Box::new(target_expr),
    }))
}

fn parse_function_def(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut name = String::new();
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::parameter => parameters.push(parse_parameter(inner)?),
            Rule::r#type => return_type = Some(parse_type(inner)?),
            Rule::block => body = parse_block(inner)?,
            _ => {}
        }
    }

    Ok(Statement::FunctionDeclaration {
        name,
        parameters,
        return_type,
        body,
    })
}

fn parse_parameter(pair: pest::iterators::Pair<Rule>) -> Result<Parameter, String> {
    let mut name = String::new();
    let mut type_annotation = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::r#type => type_annotation = Some(parse_type(inner)?),
            _ => {}
        }
    }

    Ok(Parameter {
        name,
        type_annotation,
    })
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut condition = None;
    let mut then_branch = Vec::new();
    let mut else_ifs = Vec::new();
    let mut else_branch = None;
    let mut current_else_if_condition = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => {
                if condition.is_none() {
                    condition = Some(parse_expression(inner)?);
                } else {
                    current_else_if_condition = Some(parse_expression(inner)?);
                }
            }
            Rule::block => {
                let body = parse_block(inner)?;
                if condition.is_some() && then_branch.is_empty() {
                    then_branch = body;
                } else if let Some(cond) = current_else_if_condition.take() {
                    else_ifs.push((cond, body));
                } else {
                    else_branch = Some(body);
                }
            }
            Rule::statement => {
                let stmt = parse_statement(inner)?;
                if let Some(s) = stmt {
                    if then_branch.is_empty() {
                        then_branch = vec![s];
                    } else if let Some(cond) = current_else_if_condition.take() {
                        else_ifs.push((cond, vec![s]));
                    } else {
                        else_branch = Some(vec![s]);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Statement::If {
        condition: condition.ok_or("Missing condition in if statement")?,
        then_branch,
        else_ifs,
        else_branch,
    })
}

fn parse_for_loop(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut identifiers = Vec::new();
    let mut iterable = None;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => identifiers.push(inner.as_str().to_string()),
            Rule::expression => iterable = Some(parse_expression(inner)?),
            Rule::block => body = parse_block(inner)?,
            _ => {}
        }
    }

    let (key_var, value_var) = if identifiers.len() == 2 {
        (Some(identifiers[0].clone()), identifiers[1].clone())
    } else {
        (None, identifiers[0].clone())
    };

    Ok(Statement::ForLoop {
        key_var,
        value_var,
        iterable: iterable.ok_or("Missing iterable in for loop")?,
        body,
    })
}

fn parse_while_loop(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut condition = None;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => condition = Some(parse_expression(inner)?),
            Rule::block => body = parse_block(inner)?,
            _ => {}
        }
    }

    Ok(Statement::WhileLoop {
        condition: condition.ok_or("Missing condition in while loop")?,
        body,
    })
}

fn parse_loop_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut condition = None;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::expression => condition = Some(parse_expression(inner)?),
            Rule::block => body = parse_block(inner)?,
            _ => {}
        }
    }

    Ok(Statement::Loop {
        condition: condition.ok_or("Missing condition in loop statement")?,
        body,
    })
}

fn parse_try_catch(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut try_body = Vec::new();
    let mut error_var = String::new();
    let mut catch_body = Vec::new();
    let mut is_catch = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::block => {
                if !is_catch {
                    try_body = parse_block(inner)?;
                    is_catch = true;
                } else {
                    catch_body = parse_block(inner)?;
                }
            }
            Rule::identifier => error_var = inner.as_str().to_string(),
            _ => {}
        }
    }

    Ok(Statement::TryCatch {
        try_body,
        error_var,
        catch_body,
    })
}

fn parse_return_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Statement, String> {
    let mut value = None;

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            value = Some(parse_expression(inner)?);
        }
    }

    Ok(Statement::Return(value))
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::statement {
            if let Some(stmt) = parse_statement(inner)? {
                statements.push(stmt);
            }
        }
    }

    Ok(statements)
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let inner = pair.into_inner().next();

    if inner.is_none() {
        return Err("Empty expression".to_string());
    }

    let inner = inner.unwrap();

    match inner.as_rule() {
        Rule::ternary_then => parse_ternary_then(inner),
        Rule::ternary_qmark => parse_ternary_qmark(inner),
        Rule::binary_expression => parse_binary_expression(inner),
        Rule::primary_expression => parse_primary_expression(inner),
        _ => Err(format!("Unexpected expression rule: {:?}", inner.as_rule())),
    }
}

fn parse_binary_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    // Collect expressions and operators
    let mut parts = pair.into_inner();
    let mut exprs: Vec<Expression> = Vec::new();
    let mut ops: Vec<BinaryOperator> = Vec::new();

    exprs.push(parse_primary_expression(parts.next().unwrap())?);

    while let Some(op_pair) = parts.next() {
        if op_pair.as_rule() == Rule::binary_op {
            let op = match op_pair.as_str() {
                "+" => BinaryOperator::Add,
                "-" => BinaryOperator::Subtract,
                "*" => BinaryOperator::Multiply,
                "/" => BinaryOperator::Divide,
                "%" => BinaryOperator::Modulo,
                "^" | "**" => BinaryOperator::Power,
                "==" | "=" | "===" => BinaryOperator::Equal,
                "!=" | "!==" => BinaryOperator::NotEqual,
                "<" => BinaryOperator::LessThan,
                ">" => BinaryOperator::GreaterThan,
                "<=" => BinaryOperator::LessThanOrEqual,
                ">=" => BinaryOperator::GreaterThanOrEqual,
                "is" => BinaryOperator::Is,
                "is not" => BinaryOperator::IsNot,
                "and" => BinaryOperator::And,
                "or" => BinaryOperator::Or,
                _ => return Err(format!("Unknown binary operator: {}", op_pair.as_str())),
            };

            ops.push(op);
            exprs.push(parse_primary_expression(parts.next().unwrap())?);
        }
    }

    // If no operators, return the single expression
    if ops.is_empty() {
        return Ok(exprs.remove(0));
    }

    // First, fold segments separated by logical operators (and/or)
    let mut segment_exprs: Vec<Expression> = Vec::new();
    let mut logical_ops: Vec<BinaryOperator> = Vec::new();
    let mut seg_start: usize = 0;

    for (i, op) in ops.iter().enumerate() {
        if matches!(op, BinaryOperator::And | BinaryOperator::Or) {
            // Fold exprs[seg_start..=i] left-to-right
            let mut seg = exprs[seg_start].clone();
            for k in seg_start..i {
                seg = Expression::BinaryOp {
                    left: Box::new(seg),
                    op: ops[k].clone(),
                    right: Box::new(exprs[k + 1].clone()),
                };
            }
            segment_exprs.push(seg);
            logical_ops.push(op.clone());
            seg_start = i + 1;
        }
    }

    // Fold the final segment
    let mut seg = exprs[seg_start].clone();
    for k in seg_start..ops.len() {
        // Only non-logical ops should be here; but folding is safe
        seg = Expression::BinaryOp {
            left: Box::new(seg),
            op: ops[k].clone(),
            right: Box::new(exprs[k + 1].clone()),
        };
    }
    segment_exprs.push(seg);

    // Now fold logical ops over the folded segments
    let mut result = segment_exprs[0].clone();
    for (i, seg) in segment_exprs.iter().enumerate().skip(1) {
        let lop = logical_ops[i - 1].clone();
        result = Expression::BinaryOp {
            left: Box::new(result),
            op: lop,
            right: Box::new(seg.clone()),
        };
    }

    Ok(result)
}

fn parse_primary_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let pair_str = pair.as_str();
    let pair_span = pair.as_span();
    let inner = pair.into_inner().next();

    if inner.is_none() {
        return Err(format!(
            "Empty primary expression at {:?}, content: '{}'",
            pair_span, pair_str
        ));
    }

    let inner = inner.unwrap();

    match inner.as_rule() {
        Rule::literal => parse_literal(inner),
        Rule::identifier => Ok(Expression::Identifier(inner.as_str().to_string())),
        Rule::ephemeral_var => {
            // ephemeral_var is "#" followed by identifier or digits
            // Since identifier is atomic, we need to parse from the string
            let text = inner.as_str();
            if text.starts_with('#') {
                let var_name = text[1..].trim().to_string();
                Ok(Expression::EphemeralVar(var_name))
            } else {
                Err("Invalid ephemeral variable format".to_string())
            }
        }
        Rule::ephemeral_assignment => {
            // Parse ephemeral assignment: base_expr#var_name
            let mut parts = inner.into_inner();
            let base = parts
                .next()
                .ok_or("Missing base expression in ephemeral assignment")?;
            let ephemeral = parts
                .next()
                .ok_or("Missing ephemeral variable in ephemeral assignment")?;

            // Parse the base expression
            let base_expr = match base.as_rule() {
                Rule::identifier => Expression::Identifier(base.as_str().to_string()),
                Rule::literal => parse_literal(base)?,
                _ => return Err("Unexpected ephemeral assignment base".to_string()),
            };

            // Parse the ephemeral variable name
            let ephemeral_text = ephemeral.as_str();
            let var_name = if ephemeral_text.starts_with('#') {
                ephemeral_text[1..].trim().to_string()
            } else {
                return Err("Invalid ephemeral variable format".to_string());
            };

            // Create an assignment where the ephemeral var is the target (left side)
            // and the base expression is what gets evaluated and assigned (right side)
            Ok(Expression::BinaryOp {
                left: Box::new(Expression::EphemeralVar(var_name)),
                op: BinaryOperator::Assign,
                right: Box::new(base_expr),
            })
        }
        Rule::function_call => parse_function_call(inner),
        Rule::method_call => parse_method_call(inner),
        Rule::dot_access => parse_dot_access(inner),
        Rule::bracket_access => parse_bracket_access(inner),
        _ => Err(format!(
            "Unexpected primary expression rule: {:?}",
            inner.as_rule()
        )),
    }
}

fn parse_ternary_then(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut parts = pair.into_inner();
    let condition = parse_primary_expression(parts.next().unwrap())?;
    let true_expr = parse_primary_expression(parts.next().unwrap())?;
    let false_expr = parse_primary_expression(parts.next().unwrap())?;

    Ok(Expression::TernaryThen {
        condition: Box::new(condition),
        true_expr: Box::new(true_expr),
        false_expr: Box::new(false_expr),
    })
}

fn parse_ternary_qmark(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut parts = pair.into_inner();
    let condition = parse_primary_expression(parts.next().unwrap())?;
    let true_expr = parse_primary_expression(parts.next().unwrap())?;
    let false_expr = parse_primary_expression(parts.next().unwrap())?;

    Ok(Expression::TernaryQuestion {
        condition: Box::new(condition),
        true_expr: Box::new(true_expr),
        false_expr: Box::new(false_expr),
    })
}

fn parse_literal(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::integer => {
            let s = inner.as_str();
            if s.contains('.') {
                let val = s.parse::<f64>().map_err(|_| "Invalid float")?;
                Ok(Expression::Literal(Literal::Float(val)))
            } else {
                let val = s.parse::<i64>().map_err(|_| "Invalid integer")?;
                Ok(Expression::Literal(Literal::Integer(val)))
            }
        }
        Rule::float => {
            let val = inner.as_str().parse::<f64>().map_err(|_| "Invalid float")?;
            Ok(Expression::Literal(Literal::Float(val)))
        }
        Rule::string => {
            let s = inner.as_str();
            let trimmed = &s[1..s.len() - 1]; // Remove quotes
            Ok(Expression::Literal(Literal::String(trimmed.to_string())))
        }
        Rule::string_interpolation => parse_string_interpolation(inner),
        Rule::boolean => {
            let val = inner.as_str() == "true";
            Ok(Expression::Literal(Literal::Boolean(val)))
        }
        Rule::not_defined => match inner.as_str() {
            "null" => Ok(Expression::Literal(Literal::Null)),
            "undefined" => Ok(Expression::Literal(Literal::Undefined)),
            "nil" => Ok(Expression::Literal(Literal::Nil)),
            _ => Err("Unknown not_defined value".to_string()),
        },
        Rule::regex => {
            let s = inner.as_str();
            let pattern = &s[1..s.len() - 1]; // Remove slashes
            Ok(Expression::Literal(Literal::Regex(pattern.to_string())))
        }
        Rule::array_literal => parse_array_literal(inner),
        Rule::object_literal => parse_object_literal(inner),
        Rule::pairs_literal => parse_pairs_literal(inner),
        _ => Err(format!("Unexpected literal rule: {:?}", inner.as_rule())),
    }
}

fn parse_string_interpolation(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let s = pair.as_str();
    let content = &s[1..s.len() - 1]; // Remove quotes

    let mut parts = Vec::new();
    let mut current_text = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '#' {
            if let Some(&'{') = chars.peek() {
                chars.next(); // consume '{'

                // Save any text we've accumulated
                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                // Find the closing brace
                let mut expr_str = String::new();
                let mut depth = 1;
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        depth += 1;
                        expr_str.push(ch);
                    } else if ch == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        expr_str.push(ch);
                    } else {
                        expr_str.push(ch);
                    }
                }

                // Parse the expression
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

    // Add any remaining text
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }

    Ok(Expression::StringInterpolation { parts })
}

fn parse_array_literal(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut elements = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            elements.push(parse_expression(inner)?);
        }
    }

    Ok(Expression::Literal(Literal::Array(elements)))
}

fn parse_object_literal(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let properties = HashMap::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {} // Skip the identifier name
            Rule::pairs_literal => {
                return parse_pairs_literal(inner);
            }
            _ => {}
        }
    }

    Ok(Expression::Literal(Literal::Object(properties)))
}

fn parse_pairs_literal(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
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

fn parse_function_call(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut name = String::new();
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier | Rule::base_builtin_name => name = inner.as_str().to_string(),
            Rule::expression => args.push(parse_expression(inner)?),
            Rule::builtin_function_call => return parse_function_call(inner),
            _ => {}
        }
    }

    Ok(Expression::FunctionCall { name, args })
}

fn parse_method_call(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut object = None;
    let mut method = String::new();
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                if object.is_none() {
                    object = Some(Box::new(Expression::Identifier(inner.as_str().to_string())));
                } else {
                    method = inner.as_str().to_string();
                }
            }
            Rule::string => {
                if object.is_none() {
                    let s = inner.as_str();
                    let trimmed = &s[1..s.len() - 1];
                    object = Some(Box::new(Expression::Literal(Literal::String(
                        trimmed.to_string(),
                    ))));
                }
            }
            Rule::string_interpolation => {
                if object.is_none() {
                    object = Some(Box::new(parse_string_interpolation(inner)?));
                }
            }
            Rule::array_literal => {
                if object.is_none() {
                    object = Some(Box::new(parse_array_literal(inner)?));
                }
            }
            Rule::builtin_array_method => method = inner.as_str().to_string(),
            Rule::expression => args.push(parse_expression(inner)?),
            _ => {}
        }
    }

    Ok(Expression::MethodCall {
        object: object.ok_or("Missing object in method call")?,
        method,
        args,
    })
}

fn parse_dot_access(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut object = None;
    let mut property = String::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                if object.is_none() {
                    object = Some(Box::new(Expression::Identifier(inner.as_str().to_string())));
                } else {
                    property = inner.as_str().to_string();
                }
            }
            Rule::string => {
                let s = inner.as_str();
                let trimmed = &s[1..s.len() - 1];
                object = Some(Box::new(Expression::Literal(Literal::String(
                    trimmed.to_string(),
                ))));
            }
            Rule::string_interpolation => {
                object = Some(Box::new(parse_string_interpolation(inner)?));
            }
            Rule::array_literal => {
                object = Some(Box::new(parse_array_literal(inner)?));
            }
            Rule::builtin_property_name => property = inner.as_str().to_string(),
            _ => {}
        }
    }

    Ok(Expression::PropertyAccess {
        object: object.ok_or("Missing object in property access")?,
        property,
    })
}

fn parse_bracket_access(pair: pest::iterators::Pair<Rule>) -> Result<Expression, String> {
    let mut object = None;
    let mut index = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                object = Some(Box::new(Expression::Identifier(inner.as_str().to_string())));
            }
            Rule::expression => index = Some(Box::new(parse_expression(inner)?)),
            _ => {}
        }
    }

    Ok(Expression::BracketAccess {
        object: object.ok_or("Missing object in bracket access")?,
        index: index.ok_or("Missing index in bracket access")?,
    })
}

fn parse_type(pair: pest::iterators::Pair<Rule>) -> Result<Type, String> {
    let span = pair.as_span();
    let inner = pair
        .into_inner()
        .next()
        .ok_or_else(|| format!("Empty type at {:?}", span))?;

    match inner.as_rule() {
        Rule::basic_type => parse_basic_type(&inner),
        Rule::list_type => {
            let inner_type = inner.into_inner().next().unwrap();
            Ok(Type::List(Box::new(parse_basic_type(&inner_type)?)))
        }
        Rule::pairs_type => {
            let mut types = inner.into_inner();
            let key_type = parse_basic_type(&types.next().unwrap())?;
            let value_type = parse_basic_type(&types.next().unwrap())?;
            Ok(Type::Pairs(Box::new(key_type), Box::new(value_type)))
        }
        _ => Err("Unexpected type rule".to_string()),
    }
}

fn parse_basic_type(pair: &pest::iterators::Pair<Rule>) -> Result<Type, String> {
    match pair.as_str() {
        "Int" => Ok(Type::Int),
        "Float" => Ok(Type::Float),
        "String" => Ok(Type::String),
        "Boolean" | "Bool" => Ok(Type::Boolean),
        "Array" => Ok(Type::Array),
        "Object" => Ok(Type::Object),
        "Regex" => Ok(Type::Regex),
        _ => Err(format!("Unknown type: {}", pair.as_str())),
    }
}
