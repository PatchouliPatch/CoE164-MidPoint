pub struct VarPredictor;

impl VarPredictor {
    /// Get the autocorrelation of a vector of data
    ///
    /// The function computes the first `lag`+1 autocorrelations of the
    /// provided vector of data. 
    /// The function computes the autocorrelations of the provided vector of
    /// data from `R[0]` until `R[max_lag]`. For example, if `max_lag` is 2, then
    /// the output contains three elements corresponding to R[0] until R[3],
    /// respectively
    pub fn get_autocorrelation(data: &Vec <i32>, lag: u32) -> Vec <f64> {
        let max_lag = lag as usize;
        let data_store = vec![0.0; max_lag + 1];
        if data.len() <= 1 {
            return data_store;
        }
        else {   
            for i in 0..=lag{
                let mut sum = 0.0;
                for x in 0..(data.len() - i ){
                    sum += data[x] * data[x + i];
                }
                data_store[i] = sum / (data.len() - i) as f64;
            }
        }

        data_store
    }

    /// Get the predictor coefficients
    /// 
    /// The coefficients are computed using the Levinson-Durbin algorithm.
    pub fn get_predictor_coeffs(autoc: &Vec <f64>, predictor_order: u32) -> Vec <f64> {
        let mut data_store = Vec::<f64>::new();
        //base case 
        let base_case = autoc[1]/autoc[0];
        data_store.push(base_case);
        //compute for coefficients successively, starting at i=0 until i=prediction order  - 1
        for i in 0..=(predictor_order-1){
            //Create reverse versions of the vectors data_store(coefficients) and auto correlations
            let mut a_rev = data_store.clone();
            a_rev.reverse(); 
            let mut r_ss = Vec::<f64>::new();
            for x in 0..=i{
                let y = x as usize; 
                r_ss.push(autoc[y+1])
            }
            let mut r_rev = r_ss.reverse();
            //Compute for the correction term ki+1 using a slice of the autocorrelation values and currently computed coefficients
            //dotproduct for knum ki+1,num = R(i+2) - dot(Rrev,ss[i,1], A[0, i))

            let mut dotprod = 0.0;
            let mut dotfacA = r_rev.clone();
            let mut dotfacB =data_store.clone();
            for x in 0..dotfacB.len(){
                //let y = x as usize;
                dotprod += &dotfacA[x] * &dotfacB[x];
            }
            let z = i as usize; 
            let k_num = autoc[z+2] - dotprod;

            //dot product for ki+1,den = R(0) - dot(Rss[1, i], A[0, i))
            dotprod = 0.0;
            dotfacB =data_store.clone();
            let mut dotfacAB =Vec::<f64>::new();
            if i !=0{
                for x in 0..i{
                    let y = x as usize;
                    dotfacAB.push(r_ss[y+1]);
                }
                for x in 0..dotfacAB.len(){
                    //let y = x as usize;
                    dotprod += &dotfacAB[x] * &dotfacB[x];
                }
            }
            let k_den = autoc[0] - dotprod;
            let k = k_num/k_den;
            //compute for updated coefficients
            let a_prime =  Vec::<f64>::new();
            for x in 0..=a_rev.len(){
                //let y = x as usize;
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
        //compute for shift factor first 
        //we get the maximum absolute value of coefficient 
        //iterate through lpc_coeffs and get absolute value each 
        let abs_val = lpc_coefs.iter().fold(0.0_f64, |num1, &num2| num1.max(num2.abs())); //use .fold since they are floating point
        //floor(lg(max(|L|)) + 1) - 1 = max bits
        let max_p1 = (abs_val.log2() + 1.0).floor() as u32;
        let max_bits = max_p1 -1 ;
        //compute for SF from formula sf = max(floor(pb - 1 - floor(lg(max(|L|))) ), N_SHIFT_BITS)
        let mut sf = (((precision - 1) - max_bits) as i32);

        //compute new quantized lpc
        //Initialize a rounding error variable e to zero
        let mut rounding_error = 0.0 ;
        let mut quantized = Vec::new();
        //Compute the quantized coefficient Lraw':
        //● If sf is negative, Lraw' = L / (1 << |S|)
        //● Otherwise, if sf is positive, Lraw' = L * (1 << |S|)
        for &num_coeff in lpc_coefs.iter(){
            if sf < 0 {
                let l_raw =coef / (1 << sf.abs());
            } 
            else {
                let l_raw =coef * (1 << sf);
            }
            //Compute the true quantized LPC L' with rounding error factored in L' = round(Lraw' + e)
            let l_quantized = (l_raw + e).round();
            //update the rounding error 
            e = e + (l_raw - l_quantized);
            //push into vec as u32 since output is u32 
            quantized.push(l_quantized as u32);

        }

        //If sf is negative, set the LPC shift to zero. Otherwise, leave sf as is.
        if sf < 0 {
            sf = 0;
        }

        return (quantized, sf as u32);
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
        
        //bps == bit depth 
       if bps < 16 {
            if (2+ (bps/2)) > 1{
                return (2 + (bps/2)) as u32;
            }
            else{
                return 1 as u32;
            }
       }
       else if bps == 16{
            match block_size{
                192 => {
                    return 7 as u32;
                }
                384 => {
                    return 8 as u32;
                }
                576 =>{
                    return 9 as u32;
                }
                1152 =>{
                    return 10 as u32;
                }
                2304 =>{
                    return 11 as u32;
                }
                4608 =>{
                    return 12 as u32;
                }
                _anyval =>{
                    return 13 as u32;
                }
            }
       }
       else if bps > 16{
            match block_size{
                384 => {
                    return 12 as u32;
                }
                1152 => {
                    return 13 as u32;
                }
                _anyval => {
                    return 14 as u32;
                }

            }
       }
       else { //invalud
            return 16 as u32;
       }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_01() {
        //let in_val = 0;
        let out_val_ans = 12;
        let out_val = get_best_precision(17, 384);

        assert_eq!(out_val_ans, out_val);
    }

    #[test]
    fn sample_02() {
        //let in_val = 0x164;
        let out_val_ans = 6;
        let out_val = get_best_precision(8, 1152);

        assert_eq!(out_val_ans, out_val);
    }

    #[test]
    fn sample_03() {
        //let in_val = 0x164;
        let out_val_ans = 11;
        let out_val = get_best_precision(16, 2304);

        assert_eq!(out_val_ans, out_val);
    }

    #[test] //quantized 
    fn sample_04() {

        let mut in_val = vec!{ 1.27123, -0.85145, 0.28488};
        let pb = 6 as u32;
        let (out_val,sf) = quantize_coeffs(&in_val, pb);
        
        assert_eq!(out_val[0], 20);
        assert_eq!(out_val[1], 13);
        assert_eq!(out_val[2], 4);
        assert_eq!(sf, 4);
    }
}
