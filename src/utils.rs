use time::Instant;

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

#[derive(Copy, Clone)]
pub struct Timer {
    last_instant: Instant,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            last_instant: Instant::now(),
        }
    }

    /// Marks a new tick time and returns the time elapsed in milliseconds since
    /// the last call to tick().
    pub fn tick(&mut self) -> u64 {
        let n = self.elapsed();
        self.last_instant = Instant::now();
        n
    }

    /// Returns the time elapsed in milliseconds since the last call to tick().
    pub fn elapsed(self) -> u64 {
        self.last_instant.elapsed().whole_milliseconds() as u64
    }
}

//=============================================================

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
