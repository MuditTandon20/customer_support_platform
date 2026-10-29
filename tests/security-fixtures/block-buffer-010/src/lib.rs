#[cfg(test)]
mod tests {
    use block_buffer::{BlockBuffer, Eager};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use typenum::U4;

    #[test]
    fn advisory_inline_position_corruption_does_not_reproduce_in_010() {
        // Same partial block + attacker-controlled cursor byte + caught
        // compression panic as the upstream GHSA-qwgh-2vcv-g2f7 reproduction.
        // In 0.10 the cursor is a separate field, not a byte in the buffer.
        let mut buffer = BlockBuffer::<U4, Eager>::new(&[1, 2]);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            buffer.digest_blocks(&[3, 0xff], |_| panic!("advisory reproduction"));
        })).is_err());
        assert!(buffer.get_pos() < 4);
        buffer.digest_blocks(&[3, 4], |_| {});
        assert!(buffer.get_pos() < 4);
    }
}
