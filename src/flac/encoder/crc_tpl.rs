/// Represents a kind of CRC encoding
/// 
/// This struct is used to configure the type of CRC encoding to use.
/// For example, if the generator polynomial for a CRC8 encoding is:
/// 
/// `x^8 + x^2 + x^1 + 1`
/// 
/// Then, the value of `poly` should be 0b0000_0111 (note the missing
/// MSB `1` bit) and `poly_len` should be `u8`.
pub struct CrcOptions <T> {
    poly: T,
    poly_len: T,
}


impl <T> CrcOptions <T> {
    /// Create a builder to the CRC encoder
    pub fn new(poly: T, poly_len: T) -> Self {
        CrcOptions {
            poly: poly,
            poly_len: poly_len,
        }
    }
}

impl CrcOptions <u8> {
    /// Encode data using CRC8 encoding
    /// 
    /// This method is available only if `CrcOptions` is of type `u8`.
    pub fn build_crc8(&self, data: &Vec <u8>) -> u8 {
        //create divisor for XOR ops type int for easie operations
        let divisor :i64 = ((0b1 << self.poly_len) + self.poly as i64);

        //combine all data in VEC as single number
        let mut comb_vec : i128= 0;
        
        for i in 0..data.len(){
            if i == 0{
                comb_vec = data[i] as i128;
            }
            else{
                comb_vec = comb_vec + (data[i] as i128);
            }
            comb_vec = comb_vec << self.poly_len;
        }
        //count amount of bits then shift the data by poly.len number of zeroes 
        let mut counter :u32 = comb_vec.ilog2() as u32;
        comb_vec = comb_vec << self.poly_len;
        //make a dividend base on comb_vec with poly_len number of bits
        let mut dividend :i64 = (comb_vec >> (counter)) as i64;

        //Divide or XOR continously 
        while counter != 0 {
            if dividend.ilog2() >= (self.poly_len as u32){
                //if bits of dividend is equal to bits of divisor proceed with XOR
                dividend = divisor ^ dividend;
            }
            else{
                //else append a bit from temp to the divisor 
                dividend = dividend << 1;//shift 1 to append 1 trailling bit 
                let append :i64 = ((comb_vec  & (0b1 <<(counter - 1))) >> counter -1) as i64; //get the nearest trailing bit by getting the counter-1 bit
                //combine 
                dividend = dividend + append;
                //take count of the added bit 
                counter = counter - 1;
            }
        }
        // at last append possibility that divident and divisor bits are still equal 

        if dividend.ilog2() >= (self.poly_len as u32){
            //if bits of dividend is equal to bits of divisor proceed with XOR
            dividend = divisor ^ dividend;
            return (dividend as u8)
        }
        else{
            return (dividend as u8)
        }

    }
}

impl CrcOptions <u16> {
    /// Encode data using CRC16 encoding
    /// 
    /// This method is available only if `CrcOptions` is of type `u16`.
    pub fn build_crc16(&self, data: &Vec <u16>) -> u16 {
        let divisor :i64 = ((0b1 << self.poly_len) + self.poly as i64);

        //combine all data in VEC as single number
        let mut comb_vec : i128= 0;
        
        for i in 0..data.len(){
            if i == 0{
                comb_vec = data[i] as i128;
            }
            else{
                comb_vec = comb_vec + (data[i] as i128);
            }
            comb_vec = comb_vec << self.poly_len;
        }
        //count amount of bits then shift the data by poly.len number of zeroes 
        let mut counter :u32 = comb_vec.ilog2() as u32;
        comb_vec = comb_vec << self.poly_len;
        //make a dividend base on comb_vec with poly_len number of bits
        let mut dividend :i64 = (comb_vec >> (counter )) as i64;

        //Divide or XOR continously 
        while counter != 0 {
            if dividend.ilog2() >= (self.poly_len as u32){
                //if bits of dividend is equal to bits of divisor proceed with XOR
                dividend = divisor ^ dividend;
            }
            else{
                //else append a bit from temp to the divisor 
                dividend = dividend << 1;//shift 1 to append 1 trailling bit 
                let append :i64 = ((comb_vec  & (0b1 <<counter - 1)) >> counter -1) as i64; //get the nearest trailing bit by getting the counter-1 bit
                //combine 
                dividend = dividend + append;
                //take count of the added bit 
                counter = counter - 1;
            }
        }
        // at last append possibility that divident and divisor bits are still equal 

        if dividend.ilog2() >= (self.poly_len as u32){
            //if bits of dividend is equal to bits of divisor proceed with XOR
            dividend = divisor ^ dividend;
            return (dividend as u16)
        }
        else{
            return (dividend as u16)
        }

    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_crc8_01() {
        let in_vec = vec![
            0x10,
        ];
        let ans = CrcOptions::new(0b0000_0111u8, 8)
            .build_crc8(&in_vec);

        assert_eq!(ans, 0x70);
    }

    #[test]
    fn sample_crc8_ietf_01() {
        let in_vec = vec![
            0xff, 0xf8, 0x69, 0x18,
            0x00, 0x00,
        ];
        let ans = CrcOptions::new(0b0000_0111u8, 8)
            .build_crc8(&in_vec);

        assert_eq!(ans, 0xbf);
    }

    #[test]
    fn sample_crc16_01() {
        let in_vec = vec![
            0x10, 0x00,
        ];
        let ans = CrcOptions::new(0b1000_0000_0000_0101u16, 16)
            .build_crc16(&in_vec);

        assert_eq!(ans, 0xe003);
    }

    #[test]
    fn sample_crc16_ietf_01() {
        let in_vec = vec![
            0xff, 0xf8, 0x69, 0x18,
            0x00, 0x00, 0xbf, 0x03,
            0x58, 0xfd, 0x03, 0x12,
            0x8b,
        ];
        let ans = CrcOptions::new(0b1000_0000_0000_0101u16, 16)
            .build_crc16(&in_vec);

        assert_eq!(ans, 0xaa9a);
    }
}    
