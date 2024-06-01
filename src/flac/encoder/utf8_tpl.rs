pub struct Utf8Encoder;

impl Utf8Encoder {
    /// Encode a number into its UTF-9 equivalent encoding
    /// 
    /// Although UTF-8 encoding is for characters, characters are
    /// mapped to certain numbers.
    pub fn encode(mut num: u64) -> Vec <u8> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_01() {
        let in_val = 0;
        let out_val_ans = vec![0u8];
        let out_val = Utf8Encoder::encode(in_val);

        assert_eq!(out_val_ans, out_val);
    }

    #[test]
    fn sample_02() {
        let in_val = 0x164;
        let out_val_ans = vec![0xc5u8, 0xa4u8];
        let out_val = Utf8Encoder::encode(in_val);

        assert_eq!(out_val_ans, out_val);
    }
}