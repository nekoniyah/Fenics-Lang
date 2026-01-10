use crate::ast::Expression;
use crate::features::Value;
use crate::interpreter::Interpreter;

impl Interpreter {
    pub(crate) fn call_function(
        &mut self,
        name: &str,
        args: &[Expression],
    ) -> Result<Value, String> {
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

                        self.locals.push(std::collections::HashMap::new());
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

    pub(crate) fn call_method(
        &mut self,
        obj: &Value,
        method: &str,
        args: &[Expression],
    ) -> Result<Value, String> {
        match (obj, method) {
            (Value::BridgeModule(module_name), m) => {
                let mut eval_args = Vec::new();
                for a in args {
                    eval_args.push(self.evaluate_expression(a)?);
                }
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
                        sorted.sort_by(|a, b| {
                            let na = match a {
                                Value::Integer(i) => *i as f64,
                                Value::Float(f) => *f,
                                _ => return std::cmp::Ordering::Greater,
                            };
                            let nb = match b {
                                Value::Integer(i) => *i as f64,
                                Value::Float(f) => *f,
                                _ => return std::cmp::Ordering::Less,
                            };
                            na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        for v in &sorted {
                            match v {
                                Value::Integer(_) | Value::Float(_) => {}
                                _ => return Err("sort('0-9') requires numeric array".to_string()),
                            }
                        }
                        Ok(Value::Array(sorted))
                    }
                    "9-0" => {
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
                        sorted.sort_by(|a, b| {
                            let sa = match a {
                                Value::String(s) => s,
                                _ => return std::cmp::Ordering::Greater,
                            };
                            let sb = match b {
                                Value::String(s) => s,
                                _ => return std::cmp::Ordering::Less,
                            };
                            sa.cmp(sb)
                        });
                        for v in &sorted {
                            match v {
                                Value::String(_) => {}
                                _ => return Err("sort('a-z') requires string array".to_string()),
                            }
                        }
                        Ok(Value::Array(sorted))
                    }
                    "z-a" => {
                        sorted.sort_by(|a, b| {
                            let sa = match a {
                                Value::String(s) => s,
                                _ => return std::cmp::Ordering::Less,
                            };
                            let sb = match b {
                                Value::String(s) => s,
                                _ => return std::cmp::Ordering::Greater,
                            };
                            sb.cmp(sa)
                        });
                        for v in &sorted {
                            match v {
                                Value::String(_) => {}
                                _ => return Err("sort('z-a') requires string array".to_string()),
                            }
                        }
                        Ok(Value::Array(sorted))
                    }
                    _ => {
                        Err("Unsupported sort order. Use '0-9', '9-0', 'a-z', or 'z-a'".to_string())
                    }
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

    pub(crate) fn call_function_value(
        &mut self,
        func: &Value,
        args: &[Expression],
    ) -> Result<Value, String> {
        match func {
            Value::Function { params, body } => {
                if args.len() != params.len() {
                    return Err(format!(
                        "Function takes {} arguments, but {} provided",
                        params.len(),
                        args.len()
                    ));
                }
                self.locals.push(std::collections::HashMap::new());
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
}
