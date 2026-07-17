use serde_json::Value;

pub fn resolve_path<'v>(value: &'v Value, path: &str) -> Option<&'v Value> {
    if path.is_empty() || path == "." {
        return Some(value);
    }
    let trimmed = path.strip_prefix('.').unwrap_or(path);
    if trimmed.is_empty() {
        return Some(value);
    }
    let mut current = value;
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            i += 1;
            let mut idx_str = String::new();
            while i < bytes.len() && bytes[i] != b']' {
                idx_str.push(bytes[i] as char);
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            let idx: usize = idx_str.parse().ok()?;
            current = current.get(idx)?;
        } else {
            let mut key = String::new();
            while i < bytes.len() && bytes[i] != b'.' && bytes[i] != b'[' {
                key.push(bytes[i] as char);
                i += 1;
            }
            if !key.is_empty() {
                current = current.get(&key)?;
            }
            if i < bytes.len() && bytes[i] == b'.' {
                i += 1;
            }
        }
    }
    Some(current)
}

pub fn resolve_template(template: &str, context: Option<&Value>) -> String {
    let Some(ctx) = context else {
        return template.to_string();
    };
    let mut result = String::new();
    let mut in_braces = false;
    let mut path = String::new();
    for ch in template.chars() {
        match ch {
            '{' if !in_braces => {
                in_braces = true;
                path.clear();
            }
            '}' if in_braces => {
                in_braces = false;
                let resolved = if path.is_empty() || path == "." {
                    Some(ctx)
                } else {
                    resolve_path(ctx, &path)
                };
                match resolved {
                    Some(Value::String(s)) => result.push_str(s),
                    Some(v) => result.push_str(&v.to_string()),
                    None => result.push_str(&format!("{{{{unresolved:{}}}}}", path)),
                }
            }
            c if in_braces => path.push(c),
            c => result.push(c),
        }
    }
    if in_braces {
        result.push('{');
        result.push_str(&path);
    }
    result
}

pub fn extract(value: &Value, path: &str) -> Option<String> {
    resolve_path(value, path).map(|v| match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".to_string(),
        other => other.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dot_field() {
        let v = json!({"name": "Alice"});
        assert_eq!(resolve_path(&v, ".name").unwrap(), "Alice");
    }

    #[test]
    fn test_dot_nested() {
        let v = json!({"user": {"name": "Alice"}});
        assert_eq!(resolve_path(&v, ".user.name").unwrap(), "Alice");
    }

    #[test]
    fn test_array_index() {
        let v = json!([10, 20, 30]);
        assert_eq!(resolve_path(&v, "[1]").unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_array_in_object() {
        let v = json!({"users": [{"id": 1}, {"id": 2}]});
        assert_eq!(resolve_path(&v, ".users[0].id").unwrap().as_i64(), Some(1));
    }

    #[test]
    fn test_deep_nested() {
        let v = json!({"a": {"b": [{"c": [1, 2]}]}});
        assert_eq!(resolve_path(&v, ".a.b[0].c[1]").unwrap().as_i64(), Some(2));
    }

    #[test]
    fn test_root() {
        let v = json!(42);
        assert_eq!(resolve_path(&v, ".").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_empty_path() {
        let v = json!("hello");
        assert_eq!(resolve_path(&v, "").unwrap().as_str(), Some("hello"));
    }

    #[test]
    fn test_missing_field() {
        let v = json!({"name": "Alice"});
        assert!(resolve_path(&v, ".age").is_none());
    }

    #[test]
    fn test_out_of_bounds() {
        let v = json!([1, 2]);
        assert!(resolve_path(&v, "[5]").is_none());
    }

    #[test]
    fn test_template_simple() {
        let ctx = json!({"id": 42, "name": "Alice"});
        assert_eq!(resolve_template("/users/{.id}", Some(&ctx)), "/users/42");
        assert_eq!(resolve_template("/users/{.name}", Some(&ctx)), "/users/Alice");
    }

    #[test]
    fn test_template_no_context() {
        assert_eq!(resolve_template("/users/{.id}", None), "/users/{.id}");
    }

    #[test]
    fn test_template_no_braces() {
        let ctx = json!({"id": 42});
        assert_eq!(resolve_template("/users/42", Some(&ctx)), "/users/42");
    }

    #[test]
    fn test_template_missing_path() {
        let ctx = json!({"id": 42});
        let result = resolve_template("/users/{.name}", Some(&ctx));
        assert!(result.contains("unresolved"));
    }

    #[test]
    fn test_extract_string() {
        let v = json!({"name": "Alice"});
        assert_eq!(extract(&v, ".name").unwrap(), "Alice");
    }

    #[test]
    fn test_extract_number() {
        let v = json!({"id": 42});
        assert_eq!(extract(&v, ".id").unwrap(), "42");
    }

    #[test]
    fn test_extract_object() {
        let v = json!({"user": {"name": "Alice"}});
        assert_eq!(extract(&v, ".user").unwrap(), r#"{"name":"Alice"}"#);
    }

    #[test]
    fn test_extract_missing() {
        let v = json!({"name": "Alice"});
        assert!(extract(&v, ".age").is_none());
    }
}
