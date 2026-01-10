use crate::features::Value;
use serde_json as json;
use std::collections::HashMap;

/// Bridge trait: Rust modules implement this to expose methods to Fenics
pub trait Bridge {
    fn call(&self, method: &str, args: &[Value]) -> Result<Value, String>;
}

/// Basic filesystem bridge: fs.read(path), fs.exists(path), fs.write(path, content)
pub struct FsBridge;

impl FsBridge {
    pub fn new() -> Self {
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
            _ => Err(format!(
                "Unknown fs method '{}'. Supported: read, exists, write",
                method
            )),
        }
    }
}

/// HTTP bridge: http.get(url), http.get_json(url), http.post(url, body)
pub struct HttpBridge;

impl HttpBridge {
    pub fn new() -> Self {
        Self
    }

    fn expect_string(arg: &Value, pos: usize) -> Result<String, String> {
        match arg {
            Value::String(s) => Ok(s.clone()),
            _ => Err(format!("Argument {} must be a string", pos)),
        }
    }

    fn json_to_value(v: &json::Value) -> Value {
        match v {
            json::Value::Null => Value::Null,
            json::Value::Bool(b) => Value::Boolean(*b),
            json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Float(0.0)
                }
            }
            json::Value::String(s) => Value::String(s.clone()),
            json::Value::Array(arr) => Value::Array(arr.iter().map(Self::json_to_value).collect()),
            json::Value::Object(map) => {
                let mut out = HashMap::new();
                for (k, v) in map.iter() {
                    out.insert(k.clone(), Self::json_to_value(v));
                }
                Value::Object(out)
            }
        }
    }
}

impl Bridge for HttpBridge {
    fn call(&self, method: &str, args: &[Value]) -> Result<Value, String> {
        match method {
            "get" => {
                if args.len() != 1 {
                    return Err("http.get(url) takes exactly 1 argument".to_string());
                }
                let url = Self::expect_string(&args[0], 1)?;
                let resp =
                    reqwest::blocking::get(&url).map_err(|e| format!("http.get error: {}", e))?;
                let text = resp
                    .text()
                    .map_err(|e| format!("http.get read error: {}", e))?;
                Ok(Value::String(text))
            }
            "get_json" => {
                if args.len() != 1 {
                    return Err("http.get_json(url) takes exactly 1 argument".to_string());
                }
                let url = Self::expect_string(&args[0], 1)?;
                let resp = reqwest::blocking::get(&url)
                    .map_err(|e| format!("http.get_json error: {}", e))?;
                let v: json::Value = resp
                    .json()
                    .map_err(|e| format!("http.get_json parse error: {}", e))?;
                Ok(Self::json_to_value(&v))
            }
            "post" => {
                if args.len() != 2 {
                    return Err("http.post(url, body) takes exactly 2 arguments".to_string());
                }
                let url = Self::expect_string(&args[0], 1)?;
                let body = Self::expect_string(&args[1], 2)?;
                let client = reqwest::blocking::Client::new();
                let resp = client
                    .post(&url)
                    .body(body)
                    .send()
                    .map_err(|e| format!("http.post error: {}", e))?;
                let text = resp
                    .text()
                    .map_err(|e| format!("http.post read error: {}", e))?;
                Ok(Value::String(text))
            }
            _ => Err(format!(
                "Unknown http method '{}'. Supported: get, get_json, post",
                method
            )),
        }
    }
}
