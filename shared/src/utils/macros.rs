#[macro_export]
macro_rules! fuzzy_compare {
    ($a:expr, $b:expr, $t:expr) => {
        ($a - $b).abs() <= $t
    }
}