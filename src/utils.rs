pub fn digit_count(mut n: u32, b: u32) -> u32 {
    let mut d = 0;
    loop {
        n /= b;
        d += 1;
        if n == 0 {
            return d;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_count_base_10() {
        assert_eq!(digit_count(0, 10), 1);
        assert_eq!(digit_count(9, 10), 1);
        assert_eq!(digit_count(10, 10), 2);
        assert_eq!(digit_count(99, 10), 2);
        assert_eq!(digit_count(100, 10), 3);
        assert_eq!(digit_count(999, 10), 3);
        assert_eq!(digit_count(1000, 10), 4);
        assert_eq!(digit_count(9999, 10), 4);
        assert_eq!(digit_count(10000, 10), 5);
        assert_eq!(digit_count(99999, 10), 5);
        assert_eq!(digit_count(100000, 10), 6);
        assert_eq!(digit_count(999999, 10), 6);
        assert_eq!(digit_count(1000000, 10), 7);
        assert_eq!(digit_count(9999999, 10), 7);
    }
}
