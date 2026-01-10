use crate::ast::{BinaryOperator, UnaryOperator};
use crate::features::Value;
use crate::interpreter::Interpreter;

impl Interpreter {
    pub fn evaluate_binary_op(
        &self,
        left: &Value,
        op: &BinaryOperator,
        right: &Value,
    ) -> Result<Value, String> {
        match op {
            // Arithmetic
            BinaryOperator::Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                _ => Err("Invalid types for addition".to_string()),
            },
            BinaryOperator::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err("Invalid types for subtraction".to_string()),
            },
            BinaryOperator::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err("Invalid types for multiplication".to_string()),
            },
            BinaryOperator::Divide => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if *b == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Integer(a / b))
                    }
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                _ => Err("Invalid types for division".to_string()),
            },
            BinaryOperator::Modulo => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a % b)),
                _ => Err("Modulo only supports integers".to_string()),
            },
            BinaryOperator::Power => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    Ok(Value::Float((*a as f64).powf(*b as f64)))
                }
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.powf(*b as f64))),
                _ => Err("Invalid types for power".to_string()),
            },

            // Comparison
            BinaryOperator::Equal => Ok(Value::Boolean(self.values_equal(left, right))),
            BinaryOperator::NotEqual => Ok(Value::Boolean(!self.values_equal(left, right))),
            BinaryOperator::LessThan => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) < *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a < (*b as f64))),
                _ => Err("Invalid types for comparison".to_string()),
            },
            BinaryOperator::GreaterThan => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) > *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a > (*b as f64))),
                _ => Err("Invalid types for comparison".to_string()),
            },
            BinaryOperator::LessThanOrEqual => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) <= *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a <= (*b as f64))),
                _ => Err("Invalid types for comparison".to_string()),
            },
            BinaryOperator::GreaterThanOrEqual => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Integer(a), Value::Float(b)) => Ok(Value::Boolean((*a as f64) >= *b)),
                (Value::Float(a), Value::Integer(b)) => Ok(Value::Boolean(*a >= (*b as f64))),
                _ => Err("Invalid types for comparison".to_string()),
            },
            BinaryOperator::Is => Ok(Value::Boolean(self.values_equal(left, right))),
            BinaryOperator::IsNot => Ok(Value::Boolean(!self.values_equal(left, right))),
            BinaryOperator::Match | BinaryOperator::NotMatch => {
                Err("Regex matching not yet implemented".to_string())
            }

            // Logical
            BinaryOperator::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
            BinaryOperator::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),

            // Assignments should not reach here
            _ => Err("Invalid binary operator".to_string()),
        }
    }

    pub fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Integer(a), Value::Float(b)) => *a as f64 == *b,
            (Value::Float(a), Value::Integer(b)) => *a == *b as f64,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    pub fn evaluate_unary_op(&self, op: &UnaryOperator, operand: &Value) -> Result<Value, String> {
        match (op, operand) {
            (UnaryOperator::Not, val) => Ok(Value::Boolean(!val.is_truthy())),
            (UnaryOperator::Negate, Value::Integer(i)) => Ok(Value::Integer(-i)),
            (UnaryOperator::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
            _ => Err("Unary operation not supported".to_string()),
        }
    }
}
