use crate::ast::*;
use crate::features::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;

impl Interpreter {
    pub fn execute_statement(&mut self, statement: &Statement) -> Result<Option<Value>, String> {
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

            Statement::LibExport { name, exports } => self.handle_lib_export(name, exports),

            Statement::Import { path, alias } => self.handle_import(path, alias),

            Statement::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(None)
            }
        }
    }

    fn handle_lib_export(
        &mut self,
        name: &str,
        exports: &[String],
    ) -> Result<Option<Value>, String> {
        let mut map: HashMap<String, Value> = HashMap::new();
        for fname in exports {
            if let Some(Value::Function { params, body }) = self.globals.get(fname.as_str()) {
                map.insert(
                    fname.clone(),
                    Value::Function {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            } else {
                return Err(format!("Export '{}' not found or not a function", fname));
            }
        }
        self.globals.insert(name.to_string(), Value::Object(map));
        Ok(None)
    }

    fn handle_import(
        &mut self,
        path: &str,
        alias: &Option<String>,
    ) -> Result<Option<Value>, String> {
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

        let register_name = alias.clone().or(lib_name.clone()).ok_or_else(|| {
            "Imported file does not declare a lib export; use 'as' to name it".to_string()
        })?;

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
}
