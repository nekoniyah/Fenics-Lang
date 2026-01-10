use crate::ast::*;
use crate::features::Value;
use crate::interpreter::Interpreter;
use crate::utils::string_interpolation::evaluate_string_parts;

impl Interpreter {
    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Literal(lit) => self.evaluate_literal(lit),

            Expression::Identifier(name) => self.get_variable(name),

            Expression::EphemeralVar(name) => self
                .ephemerals
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Ephemeral variable '{}' not found", name)),

            Expression::FunctionCall { name, args } => self.call_function(name, args),

            Expression::MethodCall {
                object,
                method,
                args,
            } => {
                let obj_value = self.evaluate_expression(object)?;
                self.call_method(&obj_value, method, args)
            }

            Expression::PropertyAccess { object, property } => {
                let obj_value = self.evaluate_expression(object)?;
                self.get_property(&obj_value, property)
            }

            Expression::BracketAccess { object, index } => {
                let obj_value = self.evaluate_expression(object)?;
                let index_value = self.evaluate_expression(index)?;
                self.get_bracket_access(&obj_value, &index_value)
            }

            Expression::BinaryOp { left, op, right } => {
                // Handle assignment operations specially
                match op {
                    BinaryOperator::Assign
                    | BinaryOperator::AddAssign
                    | BinaryOperator::SubAssign
                    | BinaryOperator::MulAssign
                    | BinaryOperator::DivAssign
                    | BinaryOperator::ModAssign => {
                        let right_val = self.evaluate_expression(right)?;
                        self.assign_value(left, op, right_val)
                    }
                    _ => {
                        let left_val = self.evaluate_expression(left)?;
                        let right_val = self.evaluate_expression(right)?;
                        self.evaluate_binary_op(&left_val, op, &right_val)
                    }
                }
            }

            Expression::UnaryOp { op, operand } => match op {
                UnaryOperator::Increment | UnaryOperator::Decrement => {
                    self.increment_decrement(operand, op)
                }
                _ => {
                    let val = self.evaluate_expression(operand)?;
                    self.evaluate_unary_op(op, &val)
                }
            },

            Expression::TernaryThen {
                condition,
                true_expr,
                false_expr,
            }
            | Expression::TernaryQuestion {
                condition,
                true_expr,
                false_expr,
            } => {
                let cond_value = self.evaluate_expression(condition)?;
                if cond_value.is_truthy() {
                    self.evaluate_expression(true_expr)
                } else {
                    self.evaluate_expression(false_expr)
                }
            }

            Expression::StringInterpolation { parts } => {
                let rendered =
                    evaluate_string_parts(parts, |expr| match self.evaluate_expression(expr) {
                        Ok(val) => Ok(val.to_string()),
                        Err(e) => {
                            if let Expression::Identifier(name) = expr {
                                if let Some(ev) = self.ephemerals.get(name) {
                                    return Ok(ev.to_string());
                                }
                            }
                            Err(e)
                        }
                    })?;

                Ok(Value::String(rendered))
            }
        }
    }
}
