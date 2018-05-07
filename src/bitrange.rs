use std::ops::Range;

pub trait BitRange {
    fn range_u8(&self, range: Range<usize>) -> u8;
    fn range_u16(&self, range: Range<usize>) -> u16;

    // TODO: Add if I need them
    // fn range_u32(&self, range: Range<usize>) -> u32;
    // fn range_u64(&self, range: Range<usize>) -> u64;
}

impl BitRange for u32 {
    fn range_u8(&self, range: Range<usize>) -> u8 {
        let num_bits = (range.end - range.start) + 1;

        assert!(num_bits > 0);
        assert!(num_bits < 32);

        let mask = 2_u32.pow(num_bits as u32) - 1;

        ((self >> range.start) & mask) as u8
    }

    fn range_u16(&self, range: Range<usize>) -> u16 {
        let num_bits = (range.end - range.start) + 1;

        assert!(num_bits > 0);
        assert!(num_bits < 32);

        let mask = 2_u32.pow(num_bits as u32) - 1;

        ((self >> range.start) & mask) as u16
    }
}

impl BitRange for u16 {
    fn range_u8(&self, range: Range<usize>) -> u8 {
        let num_bits = (range.end - range.start) + 1;

        assert!(num_bits > 0);
        assert!(num_bits < 16);

        let mask = 2_u16.pow(num_bits as u32) - 1;
        ((self >> range.start) & mask) as u8
    }

    fn range_u16(&self, range: Range<usize>) -> u16 {
        let num_bits = (range.end - range.start) + 1;

        assert!(num_bits > 0);
        assert!(num_bits < 32);

        let mask = 2_u16.pow(num_bits as u32) - 1;

        ((self >> range.start) & mask) as u16
    }
}

impl BitRange for u8 {
    fn range_u8(&self, range: Range<usize>) -> u8 {
        let num_bits = (range.end - range.start) + 1;

        assert!(num_bits > 0);
        assert!(num_bits < 8);

        let mask = 2_u8.pow(num_bits as u32) - 1;
        ((self >> range.start) & mask) as u8
    }

    fn range_u16(&self, range: Range<usize>) -> u16 {
        let num_bits = (range.end - range.start) + 1;

        assert!(num_bits > 0);
        assert!(num_bits < 8);

        let mask = 2_u8.pow(num_bits as u32) - 1;

        ((self >> range.start) & mask) as u16
    }
}
