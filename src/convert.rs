/// This function takes in the number as received from the CSV (a fractional number)
/// and converts it to an integer with 4 decimal places precision.
///
/// You've mentioned 4 decimals as the required precision for these numbers.
/// I believe the cleanest way to do return client balances with the right
/// precision is to tackle it at the entry/exit points of my system, imho.
pub fn convert_fractional_to_number(f: f64) -> u64 {
    (f * 10_000.0).round() as u64
}
