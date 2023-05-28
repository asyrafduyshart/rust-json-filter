use jaq_core::{parse, Ctx, Definitions, Error, RcIter, Val};
use jaq_std;
use serde_json::Value;

/// .
///
/// # Panics
///
/// Panics if .
///
/// # Errors
///
/// This function will return an error if .
pub fn apply(x: Value, f: &str) -> Vec<Result<Val, Error>>  {
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