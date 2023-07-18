use core::fmt;

use gjson::Value;

pub fn apply<'a>(line: &'a str, filters: &'a [&'a str]) -> Vec<bool> {
	let prs = gjson::parse(line);

	let mut result = Value::default();

	let mut bool_arr: Vec<bool> = Vec::new();

	for filter in filters {
		result = prs.get(filter);
		if result.str().is_empty() {
			bool_arr.push(false);
		} else {
			bool_arr.push(true);
		}
	}

	bool_arr
}
