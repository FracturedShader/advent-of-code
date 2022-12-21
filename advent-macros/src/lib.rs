//! # Summary
//! The primary macro here ([`advent_macros::generate_year`]) is a convenient way to select a day
//! and part solution for any implemented days using only two numbers: year and highest solved day.
//! The macro generates `use` and `mod` declarations as well as a
//! `pub fn run_solution(day: i32, part: i32)` that tries to load any input from a `data` folder
//! and passes it to the matching `day_##::part_##(reader: Option<impl BufRead>)`, if it exists,
//! and a `pub fn days_solved() -> i32` to check how many days have solutions.
//!
//!
//! # Example
//! To generate a method that will select from the first 19 days (inclusive) of the year 2015
//! simply requires calling the macro with the two numbers.
//!
//! ```
//! use advent_macros::generate_year;
//!
//! generate_year!(2015 19);
//! ```

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse_macro_input, LitInt};

struct YearInput {
    year: LitInt,
    max_day: LitInt,
}

impl Parse for YearInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            year: input.parse()?,
            max_day: input.parse()?,
        })
    }
}

/// A top-level convenience macro for avoiding year module boilerplate. This macro creates a
/// `run_solution(day: i32, part: i32)` function that takes care of matching the given day and part
/// to `day_##::part_##(reader: Option<impl BufRead>)` if such a solution exists. It also creates a
/// `days_solved() -> i32` function to see how many days have solutions. The macro expects to be
/// called with two integar literals such as `generate_year!(2015 19);` with the literals
/// representing the modules year and highest solved day (inclusive) respectively.
#[proc_macro]
pub fn generate_year(input: TokenStream) -> TokenStream {
    let YearInput { year, max_day } = parse_macro_input!(input as YearInput);

    let year: usize = year.base10_parse().expect("Year should be a usize literal");

    let max_day: usize = max_day
        .base10_parse()
        .expect("Max day should be a usize literal");

    let range = 1..=max_day;

    let day_idx = range.clone().map(syn::Index::from);
    let day_mod = range.map(|d| format_ident!("day_{:02}", d));
    let day_mod2 = day_mod.clone();

    let max_day = max_day as i32;

    let expanded = quote! {
        use std::{fs::File, io::BufReader};

        #(mod #day_mod;
            )*

        pub fn run_solution(day: i32, part: i32) {
            let reader = File::open(format!("data/{}-{:02}.txt", #year, day))
                .map(BufReader::new)
                .ok();

            match (day, part) {
                #((#day_idx, 1) => #day_mod2::part_01(reader),
                  (#day_idx, 2) => #day_mod2::part_02(reader),)*
                _ => eprintln!("No solution exists for day {} of {}", day, #year),
            }
        }

        pub fn days_solved() -> i32 {
            #max_day
        }
    };

    TokenStream::from(expanded)
}
