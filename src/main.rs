// #![allow(unused)] // For beginning only.


use prelude::*;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
	fs::File,
	io::{self, BufRead}, sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex},
};

use serde_json::Value;


use jaq_core::{parse, Ctx, Definitions, Error, RcIter, Val};
use jaq_std;

mod error;
mod prelude;
mod utils;

use std::time::Instant;
use std::io::Write;

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

    println!("Starting to filter {} lines", lines.len());

    lines.par_iter().for_each(|line| {
        total_lines.fetch_add(1, Ordering::Relaxed);
        let v: Value = serde_json::from_str(&line).expect("Unable to parse JSON");

        let output_file = Arc::clone(&output_file);  // Clone the Arc

        for fstring in &filter_strings {
            let res = filter(v.clone(), fstring);
            if res.len() > 0 && res.iter().all(|r| r.is_ok()) {
                // Write the JSON string to the file, followed by a newline character
                let mut file = output_file.lock().unwrap();
                // writeln!(file, "{}", ).unwrap();
                successful_filters.fetch_add(1, Ordering::Relaxed);
                for r in res {
                    let v = r.unwrap();
                    writeln!(file, "{}", v).unwrap();
                }
                // check if any error in res                
            }
        }
    });

    print!("Done filtering\n");

    let duration = start.elapsed();
    let duration_in_seconds = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
    let filters_per_second = total_lines.load(Ordering::Relaxed) as f64 / duration_in_seconds;
    
    println!("Time elapsed in processing is: {:?}", duration);
    println!("Total lines read: {}", total_lines.load(Ordering::Relaxed));
    println!("Successfully filtered lines: {}", successful_filters.load(Ordering::Relaxed));
    println!("Filtering speed: {:.2} lines/sec", filters_per_second);

    Ok(())
}

pub fn filter(x: Value, f: &str) -> Vec<std::result::Result<Val, Error>>  {
    let mut defs = Definitions::new(Vec::new());
    defs.insert_core();
    let mut errs = Vec::new();
    defs.insert_defs(jaq_std::std(), &mut errs);
    let f = parse::parse(&f, parse::main()).0.unwrap();
    let f = defs.finish(f, &mut errs);

    let to = |v| Val::from(v);

    let inputs = RcIter::new(core::iter::empty());
    let out: Vec<_> = f.run(Ctx::new([], &inputs), to(x)).collect();
    out
}