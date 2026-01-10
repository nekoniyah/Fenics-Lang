use crate::ast::*;
use crate::features::bridges::{Bridge, FsBridge, HttpBridge};
use crate::features::Value;
use std::collections::HashMap;

pub struct Interpreter {
    pub(crate) globals: HashMap<String, Value>,
    pub(crate) locals: Vec<HashMap<String, Value>>,
    pub(crate) ephemerals: HashMap<String, Value>,
    pub(crate) bridges: HashMap<String, Box<dyn Bridge>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interp = Self {
            globals: HashMap::new(),
            locals: Vec::new(),
            ephemerals: HashMap::new(),
            bridges: HashMap::new(),
        };

        // Register default bridges and expose them as globals
        let fs_bridge = Box::new(FsBridge::new());
        interp.bridges.insert("fs".to_string(), fs_bridge);
        interp
            .globals
            .insert("fs".to_string(), Value::BridgeModule("fs".to_string()));

        let http_bridge = Box::new(HttpBridge::new());
        interp.bridges.insert("http".to_string(), http_bridge);
        interp
            .globals
            .insert("http".to_string(), Value::BridgeModule("http".to_string()));

        interp
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), String> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
}
