use crate::ast::*;
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

pub struct Interpreter {
    globals: HashMap<String, Value>,
    locals: Vec<HashMap<String, Value>>,
    ephemerals: HashMap<String, Value>,
    bridges: HashMap<String, Box<dyn Bridge>>, 
}
// Bridge trait: Rust modules implement this to expose methods to Fenics
pub trait Bridge {
    fn call(&self, method: &str, args: &[Value]) -> Result<Value, String>;
}

// Basic filesystem bridge: fs.read(path), fs.exists(path), fs.write(path, content)
struct FsBridge;

impl FsBridge {
    fn new() -> Self {
        FsBridge
    }

    fn expect_string(arg: &Value, pos: usize) -> Result<String, String> {
        match arg {
            Value::String(s) => Ok(s.clone()),
            _ => Err(format!("Argument {} must be a string", pos)),
        }
    }
}

impl Bridge for FsBridge {
    fn call(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        match method {
            "read" => {
                if args.len() != 1 {
                    return Err("fs.read(path) takes exactly 1 argument".to_string());
                }
                let path = Self::expect_string(&args[0], 1)?;
                match std::fs::read_to_string(&path) {
                    Ok(content) => Ok(Value::String(content)),
                    Err(e) => Err(format!("fs.read error: {}", e)),
                }
            }
            "exists" => {
                if args.len() != 1 {
                    return Err("fs.exists(path) takes exactly 1 argument".to_string());
                }
                let path = Self::expect_string(&args[0], 1)?;
                Ok(Value::Boolean(std::path::Path::new(&path).exists()))
            }
            "write" => {
                if args.len() != 2 {
                    return Err("fs.write(path, content) takes exactly 2 arguments".to_string());
                }
                let path = Self::expect_string(&args[0], 1)?;
                let content = Self::expect_string(&args[1], 2)?;
                match std::fs::write(&path, content) {
                    Ok(_) => Ok(Value::Boolean(true)),
                    Err(e) => Err(format!("fs.write error: {}", e)),
                }
            }
            _ => Err(format!("Unknown fs method '{}'. Supported: read, exists, write", method)),
        }
    }
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
        interp.globals.insert("fs".to_string(), Value::BridgeModule("fs".to_string()));

        interp
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), String> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    fn execute_statement(&mut self, statement: &Statement) -> Result<Option<Value>, String> {
        match statement {
            Statement::VariableDeclaration {
                type_annotation: _,
                is_const: _,
                is_global,
                name,
                value,
            } => {
                let val = self.evaluate_expression(value)?;
                if *is_global || self.locals.is_empty() {
                    self.globals.insert(name.clone(), val);
                } else {
                    self.locals.last_mut().unwrap().insert(name.clone(), val);
                }
                Ok(None)
            }

            Statement::FunctionDeclaration {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                let func = Value::Function {
                    params: parameters.clone(),
                    body: body.clone(),
                };
                self.globals.insert(name.clone(), func);
                Ok(None)
            }

            Statement::Return(expr) => {
                if let Some(e) = expr {
                    Ok(Some(self.evaluate_expression(e)?))
                } else {
                    Ok(Some(Value::Null))
                }
            }

            Statement::If {
                condition,
                then_branch,
                else_ifs,
                else_branch,
            } => {
                let cond_value = self.evaluate_expression(condition)?;

                if cond_value.is_truthy() {
                    for stmt in then_branch {
                        if let Some(ret) = self.execute_statement(stmt)? {
                            return Ok(Some(ret));
                        }
                    }
                } else {
                    for (else_if_cond, else_if_body) in else_ifs {
                        let else_if_value = self.evaluate_expression(else_if_cond)?;
                        if else_if_value.is_truthy() {
                            for stmt in else_if_body {
                                if let Some(ret) = self.execute_statement(stmt)? {
                                    return Ok(Some(ret));
                                }
                            }
                            return Ok(None);
                        }
                    }

                    if let Some(else_body) = else_branch {
                        for stmt in else_body {
                            if let Some(ret) = self.execute_statement(stmt)? {
                                return Ok(Some(ret));
                            }
                        }
                    }
                }
                Ok(None)
            }

            Statement::ForLoop {
                key_var,
                value_var,
                iterable,
                body,
            } => {
                let iter_value = self.evaluate_expression(iterable)?;

                match iter_value {
                    Value::Array(arr) => {
                        self.locals.push(HashMap::new());
                        for (idx, val) in arr.iter().enumerate() {
                            if let Some(key) = key_var {
                                self.locals
                                    .last_mut()
                                    .unwrap()
                                    .insert(key.clone(), Value::Integer(idx as i64));
                            }
                            self.locals
                                .last_mut()
                                .unwrap()
                                .insert(value_var.clone(), val.clone());

                            for stmt in body {
                                if let Some(ret) = self.execute_statement(stmt)? {
                                    self.locals.pop();
                                    return Ok(Some(ret));
                                }
                            }
                        }
                        self.locals.pop();
                    }
                    Value::Object(obj) => {
                        self.locals.push(HashMap::new());
                        for (k, v) in obj.iter() {
                            if let Some(key) = key_var {
                                self.locals
                                    .last_mut()
                                    .unwrap()
                                    .insert(key.clone(), Value::String(k.clone()));
                            }
                            self.locals
                                .last_mut()
                                .unwrap()
                                .insert(value_var.clone(), v.clone());

                            for stmt in body {
                                if let Some(ret) = self.execute_statement(stmt)? {
                                    self.locals.pop();
                                    return Ok(Some(ret));
                                }
                            }
                        }
                        self.locals.pop();
                    }
                    _ => return Err("For loop requires an array or object".to_string()),
                }
                Ok(None)
            }

            Statement::WhileLoop { condition, body } => {
                loop {
                    let cond_value = self.evaluate_expression(condition)?;
                    if !cond_value.is_truthy() {
                        break;
                    }

                    for stmt in body {
                        if let Some(ret) = self.execute_statement(stmt)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }

            Statement::Loop { condition, body } => loop {
                let cond_value = self.evaluate_expression(condition)?;
                if cond_value.is_truthy() {
                    for stmt in body {
                        if let Some(ret) = self.execute_statement(stmt)? {
                            return Ok(Some(ret));
                        }
                    }
                }
            },

            Statement::TryCatch {
                try_body,
                error_var,
                catch_body,
            } => {
                let mut error = None;

                for stmt in try_body {
                    match self.execute_statement(stmt) {
                        Ok(Some(ret)) => return Ok(Some(ret)),
                        Ok(None) => {}
                        Err(e) => {
                            error = Some(e);
                            break;
                        }
                    }
                }

                if let Some(err_msg) = error {
                    self.locals.push(HashMap::new());
                    self.locals
                        .last_mut()
                        .unwrap()
                        .insert(error_var.clone(), Value::String(err_msg));

                    for stmt in catch_body {
                        if let Some(ret) = self.execute_statement(stmt)? {
                            self.locals.pop();
                            return Ok(Some(ret));
                        }
                    }
                    self.locals.pop();
                }
                Ok(None)
            }

            Statement::Block { expression } => {
                self.evaluate_expression(expression)?;
                Ok(None)
            }

            Statement::LibExport { name, exports } => {
                let mut map: HashMap<String, Value> = HashMap::new();
                for fname in exports {
                    if let Some(Value::Function { params, body }) = self.globals.get(fname.as_str()) {
                        map.insert(
                            fname.clone(),
                            Value::Function { params: params.clone(), body: body.clone() },
                        );
                    } else {
                        return Err(format!("Export '{}' not found or not a function", fname));
                    }
                }
                self.globals.insert(name.clone(), Value::Object(map));
                Ok(None)
            }

            Statement::Import { path, alias } => {
                let resolved_path = self.resolve_import_path(path)?;
                let source = std::fs::read_to_string(&resolved_path)
                    .map_err(|e| format!("Error reading import '{}': {}", resolved_path, e))?;
                let program = crate::parser::parse_program(&source)?;

                let mut lib_name: Option<String> = None;
                for stmt in &program.statements {
                    if let Statement::LibExport { name, .. } = stmt {
                        lib_name = Some(name.clone());
                        break;
                    }
                }

                let mut lib_interp = Interpreter::new();
                lib_interp.interpret(&program)?;

                let register_name = alias.clone().or(lib_name.clone()).ok_or_else(||
                    "Imported file does not declare a lib export; use 'as' to name it".to_string()
                )?;

                let module_value = if let Some(ref actual_name) = lib_name {
                    lib_interp
                        .globals
                        .get(actual_name)
                        .cloned()
                        .ok_or_else(|| format!("Module '{}' not found in library", actual_name))?
                } else {
                    let mut map: HashMap<String, Value> = HashMap::new();
                    for (k, v) in lib_interp.globals.iter() {
                        if let Value::Function { .. } = v {
                            map.insert(k.clone(), v.clone());
                        }
                    }
                    Value::Object(map)
                };

                self.globals.insert(register_name, module_value);
                Ok(None)
            }

            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(None)
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, String> {
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
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Text(text) => result.push_str(text),
                        StringPart::Expression(expr) => {
                            // Evaluate expression; if it's an identifier and not found,
                            // fall back to ephemeral variables for interpolation convenience.
                            let val = match self.evaluate_expression(expr) {
                                Ok(v) => v,
                                Err(e) => {
                                    if let Expression::Identifier(name) = expr.as_ref() {
                                        if let Some(ev) = self.ephemerals.get(name).cloned() {
                                            ev
                                        } else {
                                            return Err(e);
                                        }
                                    } else {
                                        return Err(e);
                                    }
                                }
                            };
                            result.push_str(&val.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
        }
    }

    fn evaluate_literal(&mut self, lit: &Literal) -> Result<Value, String> {
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

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }

        self.globals
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Variable '{}' not found", name))
    }

    fn call_function(&mut self, name: &str, args: &[Expression]) -> Result<Value, String> {
        // Built-in functions
        match name {
            "print" => {
                for arg in args {
                    let val = self.evaluate_expression(arg)?;
                    println!("{}", val.to_string());
                }
                return Ok(Value::Null);
            }
            "len" => {
                if args.len() != 1 {
                    return Err("len() takes exactly 1 argument".to_string());
                }
                let val = self.evaluate_expression(&args[0])?;
                match val {
                    Value::String(s) => Ok(Value::Integer(s.len() as i64)),
                    Value::Array(a) => Ok(Value::Integer(a.len() as i64)),
                    _ => Err("len() requires a string or array".to_string()),
                }
            }
            _ => {
                // User-defined function
                let func = self.get_variable(name)?;
                match func {
                    Value::Function { params, body } => {
                        if args.len() != params.len() {
                            return Err(format!(
                                "Function '{}' expects {} arguments, got {}",
                                name,
                                params.len(),
                                args.len()
                            ));
                        }

                        self.locals.push(HashMap::new());
                        for (param, arg) in params.iter().zip(args.iter()) {
                            let val = self.evaluate_expression(arg)?;
                            self.locals
                                .last_mut()
                                .unwrap()
                                .insert(param.name.clone(), val);
                        }

                        let mut result = Value::Null;
                        for stmt in &body {
                            if let Some(ret) = self.execute_statement(stmt)? {
                                result = ret;
                                break;
                            }
                        }

                        self.locals.pop();
                        Ok(result)
                    }
                    _ => Err(format!("'{}' is not a function", name)),
                }
            }
        }
    }

    fn call_method(
        &mut self,
        obj: &Value,
        method: &str,
        args: &[Expression],
    ) -> Result<Value, String> {
        match (obj, method) {
            // Bridge module dispatch: fs.read(), fs.write(), etc.
            (Value::BridgeModule(module_name), m) => {
                // Evaluate arguments first (avoid borrow conflict)
                let mut eval_args = Vec::new();
                for a in args {
                    eval_args.push(self.evaluate_expression(a)?);
                }
                // Lookup bridge afterwards
                match self.bridges.get(module_name) {
                    Some(bridge) => bridge.call(m, &eval_args),
                    None => Err(format!("Bridge '{}' not registered", module_name)),
                }
            }
            (Value::Array(arr), "reverse") => {
                let mut reversed = arr.clone();
                reversed.reverse();
                Ok(Value::Array(reversed))
            }
            (Value::Array(arr), "sort") => {
                if args.len() != 1 {
                    return Err("sort() takes exactly 1 argument".to_string());
                }
                let order_val = self.evaluate_expression(&args[0])?;
                let order = if let Value::String(s) = order_val {
                    s
                } else {
                    return Err("sort() requires a string order like '0-9' or 'a-z'".to_string());
                };

                let mut sorted = arr.clone();
                match order.as_str() {
                    "0-9" => {
                        // Numeric ascending
                        sorted.sort_by(|a, b| {
                            let na = match a {
                                Value::Integer(i) => *i as f64,
                                Value::Float(f) => *f,
                                _ => return std::cmp::Ordering::Greater, // non-numeric will error after
                            };
                            let nb = match b {
                                Value::Integer(i) => *i as f64,
                                Value::Float(f) => *f,
                                _ => return std::cmp::Ordering::Less,
                            };
                            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        // Validate all numeric
                        for v in &sorted {
                            match v {
                                Value::Integer(_) | Value::Float(_) => {}
                                _ => return Err("sort('0-9') requires numeric array".to_string()),
                            }
                        }
                        Ok(Value::Array(sorted))
                    }
                    "9-0" => {
                        // Numeric descending
                        sorted.sort_by(|a, b| {
                            let na = match a {
                                Value::Integer(i) => *i as f64,
                                Value::Float(f) => *f,
                                _ => return std::cmp::Ordering::Less,
                            };
                            let nb = match b {
                                Value::Integer(i) => *i as f64,
                                Value::Float(f) => *f,
                                _ => return std::cmp::Ordering::Greater,
                            };
                            nb.partial_cmp(&na).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        for v in &sorted {
                            match v {
                                Value::Integer(_) | Value::Float(_) => {}
                                _ => return Err("sort('9-0') requires numeric array".to_string()),
                            }
                        }
                        Ok(Value::Array(sorted))
                    }
                    "a-z" => {
                        // String ascending
                        sorted.sort_by(|a, b| {
                            let sa = match a { Value::String(s) => s, _ => return std::cmp::Ordering::Greater };
                            let sb = match b { Value::String(s) => s, _ => return std::cmp::Ordering::Less };
                            sa.cmp(sb)
                        });
                        for v in &sorted {
                            match v { Value::String(_) => {}, _ => return Err("sort('a-z') requires string array".to_string()) }
                        }
                        Ok(Value::Array(sorted))
                    }
                    "z-a" => {
                        // String descending
                        sorted.sort_by(|a, b| {
                            let sa = match a { Value::String(s) => s, _ => return std::cmp::Ordering::Less };
                            let sb = match b { Value::String(s) => s, _ => return std::cmp::Ordering::Greater };
                            sb.cmp(sa)
                        });
                        for v in &sorted {
                            match v { Value::String(_) => {}, _ => return Err("sort('z-a') requires string array".to_string()) }
                        }
                        Ok(Value::Array(sorted))
                    }
                    _ => Err("Unsupported sort order. Use '0-9', '9-0', 'a-z', or 'z-a'".to_string()),
                }
            }
            (Value::Array(arr), "has") => {
                if args.len() != 1 {
                    return Err("has() takes exactly 1 argument".to_string());
                }
                let search_val = self.evaluate_expression(&args[0])?;
                Ok(Value::Boolean(arr.contains(&search_val)))
            }
            (Value::String(s), "split") => {
                if args.len() != 1 {
                    return Err("split() takes exactly 1 argument".to_string());
                }
                let delimiter = self.evaluate_expression(&args[0])?;
                if let Value::String(delim) = delimiter {
                    let parts: Vec<Value> = s
                        .split(&delim)
                        .map(|p| Value::String(p.to_string()))
                        .collect();
                    Ok(Value::Array(parts))
                } else {
                    Err("split() requires a string delimiter".to_string())
                }
            }
            (Value::Object(_), "keys") => {
                if let Value::Object(obj) = obj {
                    let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::Array(keys))
                } else {
                    unreachable!()
                }
            }
            (Value::Object(map), m) => {
                if let Some(func) = map.get(m) {
                    self.call_function_value(func, args)
                } else {
                    Err(format!("Method '{}' not found", m))
                }
            }
            _ => Err(format!("Method '{}' not found", method)),
        }
    }

    fn call_function_value(&mut self, func: &Value, args: &[Expression]) -> Result<Value, String> {
        match func {
            Value::Function { params, body } => {
                if args.len() != params.len() {
                    return Err(format!(
                        "Function takes {} arguments, but {} provided",
                        params.len(),
                        args.len()
                    ));
                }
                self.locals.push(HashMap::new());
                for (p, a) in params.iter().zip(args.iter()) {
                    let val = self.evaluate_expression(a)?;
                    self.locals.last_mut().unwrap().insert(p.name.clone(), val);
                }

                let mut result = Value::Null;
                for stmt in body {
                    if let Some(ret) = self.execute_statement(stmt)? {
                        result = ret;
                        break;
                    }
                }

                self.locals.pop();
                Ok(result)
            }
            _ => Err("Target is not a function".to_string()),
        }
    }

    fn get_property(&self, obj: &Value, property: &str) -> Result<Value, String> {
        match (obj, property) {
            (Value::String(s), "length") => Ok(Value::Integer(s.len() as i64)),
            (Value::Array(arr), "length") => Ok(Value::Integer(arr.len() as i64)),
            (Value::Array(arr), "first") => arr
                .first()
                .cloned()
                .ok_or_else(|| "Array is empty".to_string()),
            (Value::Array(arr), "last") => arr
                .last()
                .cloned()
                .ok_or_else(|| "Array is empty".to_string()),
            (Value::Object(obj), prop) => obj
                .get(prop)
                .cloned()
                .ok_or_else(|| format!("Property '{}' not found", prop)),
            _ => Err(format!("Property '{}' not found", property)),
        }
    }

    fn get_bracket_access(&self, obj: &Value, index: &Value) -> Result<Value, String> {
        match (obj, index) {
            (Value::Array(arr), Value::Integer(i)) => {
                let idx = *i as usize;
                arr.get(idx)
                    .cloned()
                    .ok_or_else(|| "Index out of bounds".to_string())
            }
            (Value::Object(obj), Value::String(key)) => obj
                .get(key)
                .cloned()
                .ok_or_else(|| format!("Key '{}' not found", key)),
            _ => Err("Invalid bracket access".to_string()),
        }
    }

    fn evaluate_binary_op(
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

    fn values_equal(&self, left: &Value, right: &Value) -> bool {
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

    fn evaluate_unary_op(&self, op: &UnaryOperator, operand: &Value) -> Result<Value, String> {
        match (op, operand) {
            (UnaryOperator::Not, val) => Ok(Value::Boolean(!val.is_truthy())),
            (UnaryOperator::Negate, Value::Integer(i)) => Ok(Value::Integer(-i)),
            (UnaryOperator::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
            _ => Err("Unary operation not supported".to_string()),
        }
    }

    fn assign_value(
        &mut self,
        target: &Expression,
        op: &BinaryOperator,
        right_val: Value,
    ) -> Result<Value, String> {
        match target {
            Expression::Identifier(name) => {
                let new_val = match op {
                    BinaryOperator::Assign => right_val,
                    BinaryOperator::AddAssign => {
                        let current = self.get_variable(name)?;
                        self.evaluate_binary_op(&current, &BinaryOperator::Add, &right_val)?
                    }
                    BinaryOperator::SubAssign => {
                        let current = self.get_variable(name)?;
                        self.evaluate_binary_op(&current, &BinaryOperator::Subtract, &right_val)?
                    }
                    BinaryOperator::MulAssign => {
                        let current = self.get_variable(name)?;
                        self.evaluate_binary_op(&current, &BinaryOperator::Multiply, &right_val)?
                    }
                    BinaryOperator::DivAssign => {
                        let current = self.get_variable(name)?;
                        self.evaluate_binary_op(&current, &BinaryOperator::Divide, &right_val)?
                    }
                    BinaryOperator::ModAssign => {
                        let current = self.get_variable(name)?;
                        self.evaluate_binary_op(&current, &BinaryOperator::Modulo, &right_val)?
                    }
                    _ => return Err("Invalid assignment operator".to_string()),
                };

                // Find the variable and update it
                for scope in self.locals.iter_mut().rev() {
                    if scope.contains_key(name) {
                        scope.insert(name.clone(), new_val.clone());
                        return Ok(new_val);
                    }
                }
                if self.globals.contains_key(name) {
                    self.globals.insert(name.clone(), new_val.clone());
                    return Ok(new_val);
                }
                Err(format!("Variable '{}' not found", name))
            }
            Expression::PropertyAccess { object, property } => {
                let obj = self.evaluate_expression(object)?;
                match obj {
                    Value::Object(mut map) => {
                        let new_val = if let Some(current) = map.get(property) {
                            match op {
                                BinaryOperator::Assign => right_val,
                                BinaryOperator::AddAssign => self.evaluate_binary_op(
                                    current,
                                    &BinaryOperator::Add,
                                    &right_val,
                                )?,
                                BinaryOperator::SubAssign => self.evaluate_binary_op(
                                    current,
                                    &BinaryOperator::Subtract,
                                    &right_val,
                                )?,
                                BinaryOperator::MulAssign => self.evaluate_binary_op(
                                    current,
                                    &BinaryOperator::Multiply,
                                    &right_val,
                                )?,
                                BinaryOperator::DivAssign => self.evaluate_binary_op(
                                    current,
                                    &BinaryOperator::Divide,
                                    &right_val,
                                )?,
                                BinaryOperator::ModAssign => self.evaluate_binary_op(
                                    current,
                                    &BinaryOperator::Modulo,
                                    &right_val,
                                )?,
                                _ => return Err("Invalid assignment operator".to_string()),
                            }
                        } else {
                            right_val
                        };
                        map.insert(property.clone(), new_val.clone());
                        // TODO: Need to update the object in the original variable
                        Ok(new_val)
                    }
                    _ => Err("Can only access properties on objects".to_string()),
                }
            }
            Expression::BracketAccess { object, index } => {
                let index_val = self.evaluate_expression(index)?;
                
                // For bracket access assignment, we need to modify the original object
                match object.as_ref() {
                    Expression::Identifier(name) => {
                        // Get the object from the variable
                        let mut obj = self.get_variable(name)?;
                        
                        // Perform the assignment based on the object type
                        match (&mut obj, &index_val) {
                            (Value::Array(arr), Value::Integer(idx)) => {
                                let idx = *idx as usize;
                                if idx >= arr.len() {
                                    return Err(format!("Index {} out of bounds", idx));
                                }
                                
                                let new_val = match op {
                                    BinaryOperator::Assign => right_val,
                                    _ => {
                                        let current = &arr[idx];
                                        match op {
                                            BinaryOperator::AddAssign => self.evaluate_binary_op(current, &BinaryOperator::Add, &right_val)?,
                                            BinaryOperator::SubAssign => self.evaluate_binary_op(current, &BinaryOperator::Subtract, &right_val)?,
                                            BinaryOperator::MulAssign => self.evaluate_binary_op(current, &BinaryOperator::Multiply, &right_val)?,
                                            BinaryOperator::DivAssign => self.evaluate_binary_op(current, &BinaryOperator::Divide, &right_val)?,
                                            BinaryOperator::ModAssign => self.evaluate_binary_op(current, &BinaryOperator::Modulo, &right_val)?,
                                            _ => return Err("Invalid assignment operator".to_string()),
                                        }
                                    }
                                };
                                arr[idx] = new_val.clone();
                                
                                // Update the original variable
                                for scope in self.locals.iter_mut().rev() {
                                    if scope.contains_key(name) {
                                        scope.insert(name.clone(), obj.clone());
                                        return Ok(new_val);
                                    }
                                }
                                if self.globals.contains_key(name) {
                                    self.globals.insert(name.clone(), obj.clone());
                                    return Ok(new_val);
                                }
                                Err(format!("Variable '{}' not found", name))
                            }
                            (Value::Object(map), Value::String(key)) => {
                                let new_val = if let Some(current) = map.get(key) {
                                    match op {
                                        BinaryOperator::Assign => right_val,
                                        BinaryOperator::AddAssign => self.evaluate_binary_op(current, &BinaryOperator::Add, &right_val)?,
                                        BinaryOperator::SubAssign => self.evaluate_binary_op(current, &BinaryOperator::Subtract, &right_val)?,
                                        BinaryOperator::MulAssign => self.evaluate_binary_op(current, &BinaryOperator::Multiply, &right_val)?,
                                        BinaryOperator::DivAssign => self.evaluate_binary_op(current, &BinaryOperator::Divide, &right_val)?,
                                        BinaryOperator::ModAssign => self.evaluate_binary_op(current, &BinaryOperator::Modulo, &right_val)?,
                                        _ => return Err("Invalid assignment operator".to_string()),
                                    }
                                } else {
                                    right_val
                                };
                                map.insert(key.clone(), new_val.clone());
                                
                                // Update the original variable
                                for scope in self.locals.iter_mut().rev() {
                                    if scope.contains_key(name) {
                                        scope.insert(name.clone(), obj.clone());
                                        return Ok(new_val);
                                    }
                                }
                                if self.globals.contains_key(name) {
                                    self.globals.insert(name.clone(), obj.clone());
                                    return Ok(new_val);
                                }
                                Err(format!("Variable '{}' not found", name))
                            }
                            _ => Err("Bracket access requires an array with integer index or object with string key".to_string()),
                        }
                    }
                    _ => Err("Bracket access assignment only works on identifier variables".to_string()),
                }
            }
            Expression::EphemeralVar(name) => {
                // Ephemeral assignment: store the value and return it
                self.ephemerals.insert(name.clone(), right_val.clone());
                Ok(right_val)
            }
            _ => Err("Invalid assignment target".to_string()),
        }
    }

    fn increment_decrement(
        &mut self,
        target: &Expression,
        op: &UnaryOperator,
    ) -> Result<Value, String> {
        match target {
            Expression::Identifier(name) => {
                let current = self.get_variable(name)?;
                let new_val = match (op, &current) {
                    (UnaryOperator::Increment, Value::Integer(i)) => Value::Integer(i + 1),
                    (UnaryOperator::Decrement, Value::Integer(i)) => Value::Integer(i - 1),
                    (UnaryOperator::Increment, Value::Float(f)) => Value::Float(f + 1.0),
                    (UnaryOperator::Decrement, Value::Float(f)) => Value::Float(f - 1.0),
                    _ => return Err("Increment/decrement only works on numbers".to_string()),
                };

                // Update the variable
                for scope in self.locals.iter_mut().rev() {
                    if scope.contains_key(name) {
                        scope.insert(name.clone(), new_val.clone());
                        return Ok(new_val);
                    }
                }
                if self.globals.contains_key(name) {
                    self.globals.insert(name.clone(), new_val.clone());
                    return Ok(new_val);
                }
                Err(format!("Variable '{}' not found", name))
            }
            _ => Err("Increment/decrement only works on variables".to_string()),
        }
    }

    fn resolve_import_path(&self, path: &str) -> Result<String, String> {
        // If path contains slashes or backslashes, treat as literal path
        if path.contains('/') || path.contains('\\') {
            return Ok(path.to_string());
        }

        // Otherwise, search for module in standard locations
        let search_paths = vec![
            format!("{}.fenics", path),                    // Current dir + .fenics
            format!("libs/{}.fenics", path),              // libs/ subdirectory
            format!("../libs/{}.fenics", path),           // Parent's libs/
            format!("samples/{}.fenics", path),           // samples/ subdirectory
            format!("../samples/{}.fenics", path),        // Parent's samples/
        ];

        for candidate in search_paths {
            if std::path::Path::new(&candidate).exists() {
                return Ok(candidate);
            }
        }

        Err(format!(
            "Module '{}' not found in search paths: ./libs/, ../libs/, ./samples/, ../samples/, or current directory",
            path
        ))
    }
}
