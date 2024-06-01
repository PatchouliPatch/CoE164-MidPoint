pub struct Utf8Encoder;

impl Utf8Encoder {
    /// Encode a number into its UTF-9 equivalent encoding
    /// 
    /// Although UTF-8 encoding is for characters, characters are
    /// mapped to certain numbers.
    pub fn encode(mut num: u64) -> Vec <u8> {
        //create vec to handle the encoded and a variable for the header 
        let mut data_store = Vec::<u8>::new();
        let mut header = 0;

        //create a template to determine the number of bytes and maximum number of bits 
        let mut num_bytes =0;
        let mut max_bits =0;

        if num < 65{
            num_bytes = 0;
            max_bits = 7;
        }
        else if num < 2049 {
            num_bytes = 2;
            max_bits = 11
        }
        else if num < 65_537 {
            num_bytes = 3;
            max_bits = 16
        }
        else if num < 2_097_153{
            num_bytes = 4;
            max_bits = 21;
        }
        else if num < 67_108_865{
            num_bytes = 5;
            max_bits = 26;
        }
        else if num < 2_147_483_649{
            num_bytes = 6;
            max_bits = 31
        }
        else if num <1_099_511_627_777{
            num_bytes = 7;
            max_bits = 40;
        }
        //set up first header 
        if num_bytes != 0 {
            for i in 0..(num_bytes-1){
                header = header + 1;
                header = header << 1;
            }
        }
        //set up the rest of first byte if num_bytes not equal to 7 
        //conditional should be 7 - #header bits
        if num_bytes != 7{ 
            for i in 1..(7-num_bytes){
                let append = (num & (0b1 << (max_bits - i ))) >> (max_bits - i ); //one bit only
                header = header << 1;
                header = header + append;
            }
        }
        data_store.push(header as u8);

        if num_bytes == 0 {
            return data_store;
        }

        let mut nex_byte = 0;
        let mut reset = 0;
        //take note for preceeding bytes the first two bit is always 2b10 
        //every 6 counts we need to push the byte then reset all 
        //do this for max_bits number of times given start at first 8 bits minus the number of bytes (this is first byte)

        for i in (8-num_bytes)..=max_bits{
            let append = (num & (0b1 << (max_bits - i ))) >> (max_bits - i ); //fetch the nth bit from the front of th whole number then shift to first bit
            nex_byte = nex_byte + append;
            reset = reset + 1;

            //check if reset else shift next byte
            if reset == 6 {
                let add_ten = 0b1 << 7; //append 10 in the first two bits 
                nex_byte = nex_byte + add_ten;
                data_store.push(nex_byte as u8);
                nex_byte = 0;
                reset = 0;
            }
            else {
                nex_byte = nex_byte << 1;
            }
        }
         

        return data_store;
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
