use crate::ast::Literal;
use crate::features::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;

impl Interpreter {
    pub fn evaluate_literal(&mut self, lit: &Literal) -> Result<Value, String> {
        match lit {
            Literal::Integer(i) => Ok(Value::Integer(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Boolean(b) => Ok(Value::Boolean(*b)),
            Literal::Null | Literal::Undefined | Literal::Nil => Ok(Value::Null),
            Literal::Regex(_) => Err("Regex not yet supported".to_string()),
            Literal::Array(arr) => {
                let mut values = Vec::new();
                for expr in arr {
                    values.push(self.evaluate_expression(expr)?);
                }
                Ok(Value::Array(values))
            }
            Literal::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), self.evaluate_expression(v)?);
                }
                Ok(Value::Object(map))
            }
        }
    }
}
