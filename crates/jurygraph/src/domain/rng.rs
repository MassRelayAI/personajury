//! SplitMix64, the one small generator the seeded draw uses (no crate).
pub struct SplitMix64 {
    state: u64,
}
impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }
    pub fn from_bytes(seed: [u8; 8]) -> Self {
        SplitMix64::new(u64::from_be_bytes(seed))
    }
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A number in `0..n` (n > 0). The modulo bias is below 2^-40 for any n a panel can have.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
    /// Fisher-Yates, in place.
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_the_reference_sequence_for_seed_zero() {
        let mut r = SplitMix64::new(0);
        assert_eq!(r.next_u64(), 0xE220A8397B1DCDAF);
        assert_eq!(r.next_u64(), 0x6E789E6AA1B965F4);
        assert_eq!(r.next_u64(), 0x06C45D188009454F);
    }
    #[test]
    fn from_bytes_reads_big_endian() {
        let mut a = SplitMix64::from_bytes([0, 0, 0, 0, 0, 0, 0, 7]);
        let mut b = SplitMix64::new(7);
        assert_eq!(a.next_u64(), b.next_u64());
    }
    #[test]
    fn below_stays_in_range_and_shuffle_keeps_the_elements() {
        let mut r = SplitMix64::new(42);
        for n in 1..40 {
            assert!(r.below(n) < n);
        }
        let mut v: Vec<u32> = (0..20).collect();
        r.shuffle(&mut v);
        let mut s = v.clone();
        s.sort();
        assert_eq!(s, (0..20).collect::<Vec<_>>());
        assert_ne!(v, (0..20).collect::<Vec<_>>(), "20 elements are not left in order");
    }
}
