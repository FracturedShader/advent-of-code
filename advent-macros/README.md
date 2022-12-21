# advent-macros

## Summary
The primary macro here ([`advent_macros::generate_year`]) is a convenient way to select a day
and part solution for any implemented days using only two numbers: year and highest solved day.
The macro generates `use` and `mod` declarations as well as a
`pub fn run_solution(day: i32, part: i32)` that tries to load any input from a `data` folder
and passes it to the matching `day_##::part_##(reader: Option<impl BufRead>)`, if it exists,
and a `pub fn days_solved() -> i32` to check how many days have solutions.


## Example
To generate a method that will select from the first 19 days (inclusive) of the year 2015
simply requires calling the macro with the two numbers.

```rust
use advent_macros::generate_year;

generate_year!(2015 19);
```
