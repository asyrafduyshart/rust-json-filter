use serde_json::{Value, Number};

pub fn parse(filter_string: &str) -> Option<Vec<(&str, &str, Option<Value>, Option<&str>)>> {
    let filters = filter_string.split(" AND ").map(|filter_part| {
        let parts: Vec<&str> = filter_part.split_whitespace().collect();
        let field = parts[0].trim_start_matches('.');
        let operator = parts[1];
        let value = parts[2].trim_matches('\'');

        let value_field = if value.starts_with('.') {
            Some(value.trim_start_matches('.'))
        } else {
            None
        };
        let value = if let Ok(n) = value.parse::<i64>() {
            Some(Value::Number(Number::from(n)))
        } else if !value.starts_with('.') {
            Some(Value::String(value.to_string()))
        } else {
            None
        };
        (field, operator, value, value_field)
    }).collect();
    Some(filters)
}


pub fn apply(v: &Value, filters: &[(&str, &str, Option<Value>, Option<&str>)]) -> bool {
    filters.iter().all(|(field, operator, value, value_field)| {

        let f = v.get(*field);
        let other_field_value = value_field.and_then(|vf| v.get(vf));
        let value = value.as_ref().or(other_field_value);

        match (f, value) {
            (Some(f), Some(value)) => match *operator {
                "=" => f == value,
                "!=" => f != value,
                ">=" => f.as_i64() >= value.as_i64(),
                ">" => f.as_i64() > value.as_i64(),
                "<=" => f.as_i64() <= value.as_i64(),
                "<" => f.as_i64() < value.as_i64(),
                _ => false,  // Unknown operator
            },
            _ => false,
        }
    })
}