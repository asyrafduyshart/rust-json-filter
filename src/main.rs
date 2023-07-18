#![allow(unused)] // For beginning only.

use gjson::Value;
use prelude::*;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
	fs::File,
	io::{self, BufRead},
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc, Mutex,
	},
};

mod error;
mod filters;
mod prelude;
mod utils;

use filters::filter;
use std::io::Write;
use std::time::Instant;

use memmap::Mmap;

fn main() -> Result<()> {
	let start = Instant::now();

	let file = File::open("input.json")?;
	let mmap = unsafe { Mmap::map(&file)? };

	let filters_file = File::open("filters.jq")?;
	let filter_reader = io::BufReader::new(filters_file);
	let filter_strings: Vec<String> = filter_reader.lines().map(|l| l.unwrap()).collect();

	let output_file = File::create("output.json")?;
	let output_file = Arc::new(Mutex::new(output_file));

	let total_lines = AtomicUsize::new(0);
	let successful_filters = AtomicUsize::new(0);

	// Convert the memory-mapped data to a string (assuming it's UTF-8), split it into lines,
	// then convert that into a Vec<String>
	let data = std::str::from_utf8(&mmap[..]).expect("File data is not valid UTF-8");
	let lines: Vec<String> = data.lines().map(String::from).collect();

	lines.par_iter().for_each(|line| {
		total_lines.fetch_add(1, Ordering::Relaxed);
		filter_strings.par_iter().for_each(|filter| {
			let mut buffer: Value = Value::default(); // or however you initialize Value
			let filters: Vec<&str> = filter.split("AND").collect();
			let filters_str: Vec<&str> = filters.iter().map(|s| &s[..]).collect();

			let result = filter::apply(line, filters_str.as_slice());
			println!("filter {:?} === {:?}", filters_str, result);
			// result.array().iter().for_each(|el| {
			// 	println!("{}", el);
			// 	let mut output_file = output_file.lock().unwrap();
			// 	output_file.write_all(el.to_string().as_bytes()).unwrap();
			// 	output_file.write_all(b"\n").unwrap();
			// 	successful_filters.fetch_add(1, Ordering::Relaxed);
			// });
			// if result {
			// 	let mut output_file = output_file.lock().unwrap();
			// 	output_file.write_all(line.as_bytes()).unwrap();
			// 	output_file.write_all(b"\n").unwrap();
			// 	successful_filters.fetch_add(1, Ordering::Relaxed);
			// }
		})
	});

	println!("Done filtering");

	let duration = start.elapsed();
	let duration_in_seconds = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
	let filters_per_second = total_lines.load(Ordering::Relaxed) as f64 / duration_in_seconds;

	println!("Time elapsed in processing is: {:?}", duration);
	println!("Total lines read: {}", total_lines.load(Ordering::Relaxed));
	println!(
		"Successfully filtered lines: {}",
		successful_filters.load(Ordering::Relaxed)
	);
	println!("Filtering speed: {:.2} lines/sec", filters_per_second);

	Ok(())
}
