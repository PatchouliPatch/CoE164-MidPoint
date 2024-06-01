use std::ops::{Shl, Shr};
//use crate::flac::bitstream;

/// Represents a Rice encoder
///
/// This encoder is expected to encode `num_samples` residuals from a predictor of
/// order `predictor_order`. Note that Rice encoding in FLAC is only available
/// for LPC and FIXED audio subframes.
pub struct RiceEncoderOptions {
    num_samples: u64,
    predictor_order: u8,
}

/// Represents a Rice-encoded stream
///
/// Rice encoding is _not necessarily_ byte-aligned. The `extra_bits_len`
/// value denotes the number of LSBits in the last byte of the `stream`
/// that are _not_ part of the encoding.
#[derive(Debug)]
pub struct RiceEncodedStream {
    pub stream: Vec <u8>,
    pub param: u8,
    pub extra_bits_len: u8,
}

impl RiceEncoderOptions {
    /// Create a builder to the Rice encoder
    pub fn new(num_samples: u64, predictor_order: u8) -> Self {

        Self {
            num_samples: num_samples,
            predictor_order: predictor_order
        }

    }

    /// Get the minimum partition order
    /// 
    /// The default minimum partition order is zero
    fn min_rice_partition_order() -> u8 {
        
        0

    }

    /// Get the maximum partition order
    /// 
    /// The maximum partition order is computed as the lowest power of two
    /// that makes up the block size, or the index of the least significant
    /// 1 bit in the block size. Note that odd-sized block sizes can only
    /// have a partition order of 0 as the number of partitions should be
    /// a power of two.
    fn max_rice_partition_order(mut block_size: u64) -> u8 {

        if block_size & 2 == 2 {
            return 2;
        }

        if block_size & 4 == 4 {
            return 4;
        }

        if block_size & 8 == 8 {
            return 8;
        }

        if block_size & 16 == 16 {
            return 16;
        }

        if block_size & 32 == 32 {
            return 32;
        }

        if block_size & 64 == 64 {
            return 64;
        }

        if block_size & 128 == 128 {
            return 128;
        }

        return 1; // odd numbers

    }

    /// Compute the best partition order and best Rice parameters for each partition
    /// 
    /// The best partition order is computed based on the order that yields the minimum
    /// total number of bits of the resulting Rice encoding.
    fn best_partition_and_params(&self, residuals: &Vec <i64>) -> (Vec <u8>, u8) {
        
        
    }

    /// Compute the best Rice parameters for some partition of the residuals
    /// 
    /// The best Rice parameter `M` can be approximated using the following:
    /// 
    /// `M = log2(abs_r_mean - 1) - log2(n_partition_samples) + 1`.
    /// 
    /// Note that in practice, the sum of the absolute value of the residuals
    /// is used instead of the absolute residual mean `abs_r_mean`. In addition,
    /// Most implementations will bound `M` to be represented by at most 18 bits.
    /// 
    /// Note that only partition order 0 is allowed for odd-length residuals
    /// as the number of partitions should be a power of two.
    /// 
    /// # Errors
    /// Returns `None` if a best parameter cannot be found for any partition. This
    /// arises usually if the predictor order is larger than the amount of residuals
    /// in a partition.
    
    fn best_parameters(&self, partition_order: u8, residuals: &Vec <i64>) -> Option <(Vec <u8>, u64)> {
        
        if partition_order as usize > residuals.len() {
            return None;
        }

        let mut abs_r_mean: u64 = 0;
        let mut best_partition_order: Vec<u8> = Vec::new();

        for i in residuals.iter() {
            abs_r_mean += i.abs() as u64;
            let x: u64 = Self::zigzag(*i);
            best_partition_order.push(Self::max_rice_partition_order(x));
        }

        let logable_r_mean: f64 = abs_r_mean as f64;
        let sizeof_residuals: f64 = residuals.len() as f64;
        let parameter_M: f64 = (logable_r_mean - 1.0).log(2.0) - sizeof_residuals.log(2.0) + 1.0;
        let returnable_M: u64 = parameter_M as u64;

        return Some((best_partition_order, returnable_M));
        
    }

    /// Find the exact total number of bits needed to represent a Rice-encoded
    /// partition of samples
    /// 
    /// A residual `r` can be represented using 1 bit for the unary stop mark,
    /// `rice_param` bits for the truncated binary part of the rice encoding, and
    /// `zigzag(r) >> rice_param` bits for the unary tally marks.
    fn bits_in_partition_exact(rice_param: u8, n_partition_samples: u64, residuals: &Vec<i64>) -> u64 {
        todo!()
    }

    /// Find the total number of bits occupied by this encoding
    /// 
    /// Rice encoding uses `q + 1` bits for the unary-encoded quotient `q` and
    /// `rice_param` bits for the binary remainder
    fn bits_in_partition_sums(rice_param: u8, n_partition_samples: u64, abs_residual_sum: u64) -> u64 {
        todo!()
    }

    /// Encode residuals into Rice encoding
    /// 
    /// To encode a residual into its Rice encoding, it should be first processed
    /// using zigzag encoding so that all of the residuals become nonnegative numbers.
    /// Then, the Rice encoding of each residual is computed.
    /// 
    /// Note that the contents are _not_ ensured to be byte-aligned. Hence, this method returns
    /// the Rice-encoded byte vector containing the number of extra unused bits at the last element.
    pub fn encode(rice_param: u8, residuals: &Vec <i64>) -> RiceEncodedStream {

        /*
        
        pub struct RiceEncodedStream {
            pub stream: Vec <u8>,
            pub param: u8,
            pub extra_bits_len: u8,
        }

        */

        // use zigzag encoding to make all residuals non-negative

        let absolute_residuals: Vec<u64> = Vec::new();
        for i in residuals.iter() {
            absolute_residuals.push(Self::zigzag(*i));
        }

        /// S = residual[i]
        /// M = Rice Parameter
        /// log_2 (M) = K bits needed to represent B
        /// Evaluate U = S >> K and save result as unary
        /// B = S & (M - 1) and represent in binary padded to the left with zeros until length K
        /// Rice(S) = (U << K) | B, or U and B concatenated together;
        
        // Step 1: Get Rice Parameter M

        let data_store = Self::best_parameters(Self::RiceEncoderOptions,0, residuals);
        let rice_param: u64 = data_store[1];

    }

    /// Encode residuals into a partitioned Rice-encoded stream
    /// 
    /// This method computes the Rice encoding of a stream of residuals by first partitioning
    /// the residual into groups. Each group is then found its best Rice parameter and
    /// each residual in the group is then encoded using the parameter.
    /// 
    /// The method returns each Rice-encoded group in chronological order and the partition order,
    /// respectively. The number of elemenets in the vector of Rice-encoded groups should be less than
    /// or equal to `2^partition order`.
    /// 
    /// Note that each of the contents are _not_ ensured to be byte-aligned. Hence, this method
    /// returns the Rice-encoded byte stream and the number of extra unused bits at the last byte
    /// of the stream, respectively.
    pub fn encode_by_partition(&self, residuals: &Vec <i64>)  -> (Vec <RiceEncodedStream>, u8) {
        todo!()
    }

    /// Convert an integer into its zigzag encoding. With this encoding, all
    /// positive numbers are even and all negative numbers are odd.
    pub fn zigzag(num: i64) -> u64 { // followed the formula over at https://docs.rs/residua-zigzag/latest/zigzag/
        
        let q = (num >> 63) ^ (num << 1); 
        return q as u64;

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_sample_ietf_02() {
        let in_vec = vec![
            3194, -1297, 1228, -943,
            952, -696, 768, -524,
            599, -401, -13172, -316,
            274, -267, 134,
        ];

        let out_vec_ans = vec![
            0x11, 0xe8, 0xa2, 0x14,
            0xcc, 0x7a, 0xef, 0xb8,
            0x6b, 0x7f, 0x00, 0x60,
            0xbe, 0x57, 0x59, 0x08,
            0x00, 0x77, 0x3d, 0x3b,
            0xd1, 0x25, 0x0a, 0xc8,
            0x60,
        ];

        let rice_enc_stream = RiceEncoderOptions::encode(11, &in_vec);

        assert_eq!(rice_enc_stream.stream, out_vec_ans);
        assert_eq!(rice_enc_stream.extra_bits_len, 3);
    }

    #[test]
    fn encode_sample_ietf_03() {
        let in_vec = vec![
            3, -1, -13,
        ];

        let out_vec_ans = vec![
            0xe9, 0x12,
        ];

        let rice_enc_stream = RiceEncoderOptions::encode(3, &in_vec);

        assert_eq!(rice_enc_stream.stream, out_vec_ans);
        assert_eq!(rice_enc_stream.extra_bits_len, 1);
    }
}

fn main() {

}
