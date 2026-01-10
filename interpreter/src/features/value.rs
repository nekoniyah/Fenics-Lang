use crate::ast::{Parameter, Statement};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    // Reference to a registered Rust bridge module by name
    BridgeModule(String),
    Function {
        params: Vec<Parameter>,
        body: Vec<Statement>,
    },
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::BridgeModule(name) => format!("<bridge:{}>", name),
            Value::Function { .. } => "<function>".to_string(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Integer(0) => false,
            Value::Float(f) if *f == 0.0 => false,
            Value::String(s) if s.is_empty() => false,
            Value::Array(a) if a.is_empty() => false,
            _ => true,
        }
    }
}
