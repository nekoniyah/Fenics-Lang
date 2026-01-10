use crate::ast::{BinaryOperator, Expression, UnaryOperator};
use crate::features::Value;
use crate::interpreter::Interpreter;

impl Interpreter {
    pub(crate) fn get_variable(&self, name: &str) -> Result<Value, String> {
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

    pub(crate) fn get_property(&self, obj: &Value, property: &str) -> Result<Value, String> {
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

    pub(crate) fn get_bracket_access(&self, obj: &Value, index: &Value) -> Result<Value, String> {
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

    pub(crate) fn assign_value(
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
                        Ok(new_val)
                    }
                    _ => Err("Can only access properties on objects".to_string()),
                }
            }
            Expression::BracketAccess { object, index } => {
                let index_val = self.evaluate_expression(index)?;

                match object.as_ref() {
                    Expression::Identifier(name) => {
                        let mut obj = self.get_variable(name)?;

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
								let new_val = if let Some(current) = map.get(key.as_str()) {
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
                    _ => Err(
                        "Bracket access assignment only works on identifier variables".to_string(),
                    ),
                }
            }
            Expression::EphemeralVar(name) => {
                self.ephemerals.insert(name.clone(), right_val.clone());
                Ok(right_val)
            }
            _ => Err("Invalid assignment target".to_string()),
        }
    }

    pub(crate) fn increment_decrement(
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
}
