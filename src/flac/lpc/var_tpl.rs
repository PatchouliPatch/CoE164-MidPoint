pub struct VarPredictor;

impl VarPredictor {
    /// Get the autocorrelation of a vector of data
    ///
    /// The function computes the first `lag`+1 autocorrelations of the
    /// provided vector of data. 
    pub fn get_autocorrelation(data: &Vec <i32>, lag: u32) -> Vec <f64> {
        let data_store = Vec::<f64>::new();
        let length:usize = data.len() as usize;

        for i in 0..length{
            if length > (i+lag){
                let data_in = (data[i] * data[i+lag]) as f64;
                data_store.push(data_in);
            }
            else { //put of scope record as 0 
                data_store.push(0.0);
            }
        }

        data_store
    }

    /// Get the predictor coefficients
    /// 
    /// The coefficients are computed using the Levinson-Durbin algorithm.
    pub fn dot(autoc: &Vec <f64> , other: &Vec <f64>) -> Vec <f64> {
        autoc.vec.into_iter().zip(other.vec).fold(0_f64, |acc, elm| acc + (elm.0 * elm.1))
    }

    pub fn get_predictor_coeffs(autoc: &Vec <f64>, predictor_order: u32) -> Vec <f64> {
        let mut data_store = Vec::<f64>::new();
        //base case 
        let base_case = autoc[1]/autoc[0];
        data_store.push(base_case);
        //compute for coefficients successively, starting at i=0 until i=prediction order  - 1
        for i in i..=(predictor_order-1){
            let mut a_rev =[0; i];
            let mut r_rev =[0; i];
            //Create reverse versions of the vectors data_store(coefficients) and auto correlations
            a_rev = (data_store[0..=i]).clone();
            a_rev.reverse(); 
            r_rev = (autoc[0..=i]).clone();
            r_rev.reverse();
            //Compute for the correction term ki+1 using a slice of the autocorrelation values and currently computed coefficients
            let k_num = autoc[i+2] - dot(r_rev[0..=(r_rev.len()-1)], &data_store[0,i])
            let k_den = autoc[0] - dot(&autoc[1,i], &data_store[0,i])
            let k = k_num/k_den;
            //compute for updated coefficients
            let a_prime =  Vec::<f64>>::new();
            for x in 0..=a_rev.len(){
                a_prime.push(data_store[x]- (k*a_rev[x]));
            }
            //append kI+1 at the end 
            a_prime.push(k);
            data_store = a_prime;
        }
        data_store
    }

    /// Quantize the predictor coefficients and find their shift factor
    /// 
    /// The shift factor `S` is computed from the maximum absolute value of a coefficient
    /// `L_max`. This value is computed as `precision - lg(L_max)` or to
    /// the maximum shift value of 1 << 5 = 31, whichever is smaller. Note that it is
    /// possible for this shift factor to be negative. In that case, the shift value
    /// will still be used in quantizing the coefficients but its effective value
    /// will be zero.
    /// 
    /// Quantization involves converting the provided floating-point coefficients
    /// into integers. Each of the values are rounded up or down depending on
    /// some accummulated rounding error `\epsilon`. Initially, this error is zero.
    /// For each coefficient `L_i`, the coefficient is multiplied (for positive shift)
    /// or divided (for negative shift) by `1 << abs(S)` to get the raw value `L_i_r + \epsilon`.
    /// Then, `L_i_r + \epsilon` is rounded away from zero to get the quantized coefficient.
    /// The new rounding error `\epsilon = L_i_r + \epsilon - round(L_i_r)` is then updated for the
    /// next coefficient.
    pub fn quantize_coeffs(lpc_coefs: &Vec <f64>, mut precision: u32) -> (Vec <u32>, u32) {
        todo!()
    }

    /// Compute the residuals from a given linear predictor
    /// 
    /// The residuals are computed with the provided quantized coefficients
    /// `qlp_coefs` and shift factor `qlp_shift`.
    pub fn get_residuals(data: &Vec <i32>, qlp_coefs: &Vec <u32>, predictor_order: u32, qlp_shift: u32) -> Option <Vec <i32>> {
        todo!()
    }

    /// Get the best coefficient precision
    /// 
    /// FLAC uses the bit depth and block size to determine the best coefficient
    /// precision. By default, the precision is 14 bits but can be one of the
    /// following depending on several parameters:
    /// 
    /// | Bit depth | Block size |     Best precision      |
    /// |-----------|------------|-------------------------|
    /// |   < 16    |     any    | max(1, 2 + bit_depth/2) |
    /// |     16    |     192    |           7             |
    /// |     16    |     384    |           8             |
    /// |     16    |     576    |           9             |
    /// |     16    |    1152    |          10             |
    /// |     16    |    2304    |          11             |
    /// |     16    |    4608    |          12             |
    /// |     16    |     any    |          13             |
    /// |   > 16    |     384    |          12             |
    /// |   > 16    |    1152    |          13             |
    /// |   > 16    |     any    |          14             |
    pub fn get_best_precision(bps: u32, block_size: u32) -> u32 {
        todo!()
    }
}
