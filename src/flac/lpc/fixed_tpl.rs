pub struct FixedPredictor;

impl FixedPredictor {
    /// Get order that yields the least sum of residuals
    /// 
    /// The predictor orders are from 0 to 4 inclusive and is retrieved
    /// by finding the predictor that yields the *minimum* absolute
    /// sum of residuals for the given `data` and derived predictor.
    pub fn best_predictor_order(data: &Vec <i64>) -> Option <u8> {
        
        let mut resids: Vec<i64> = Vec::new();

        for i in 0..=4 {
            
            let value = Self::get_residuals(data, i);
            let value_arr: Vec<i64>;

            match value {
                Some(some_vec) => {
                    value_arr = some_vec;
                },
                _ => {
                    return None;
                }
            }

            let mut abs_sum: i64 = 0;

            for entry in value_arr {
                abs_sum += entry;
            }

            abs_sum = abs_sum.abs();
            resids.push(abs_sum);
        }

        let mut min = resids[0];
        let mut min_index: usize = 0;

        for q in 1..resids.len() {
            if resids[q] < min {
                min = resids[q];
                min_index = q;
            }
        }

        return Some(min_index as u8);

    }

    /// Get residuals of a fixed predictor order 
    /// 
    /// The predictor orders are from 0 to 4 inclusive and corresponds
    /// to one of the five "fixed" predictor orders written in the FLAC
    /// specification. The predictor orders are defined as follows:
    /// 
    /// 0: r[i] = 0
    /// 1: r[i] = data[i - 1]
    /// 2: r[i] = 2 * data[i - 1] - data[i - 2]
    /// 3: r[i] = 3 * data[i - 1] - 3 * data[i - 2] + data[i - 3]
    /// 4: r[i] = 4 * data[i - 1] - 6 * data[i - 2] + 4 data[i - 3] - data[i - 4]
    /// 
    /// This function returns a vector with each element containing data[i] - r[i].
    /// 
    /// # Errors
    /// `None` is returned if an error occurs in the function. This includes whether
    /// the predictor order provided is not within 0 and 4 inclusive and whether the
    /// size of `data` is less than the predictor order.
    pub fn get_residuals(data: &Vec <i64>, predictor_order: u8) -> Option <Vec <i64>> {

        if data.len() < predictor_order as usize {
            return None;
        }

        if predictor_order > 4 {
            return None;
        } 

        let mut return_data = data.clone();
        match predictor_order {

            0 => {
                return Some(return_data); // x - 0 = x
            },

            1 => {

                if data.len() < 2 {
                    return None;
                }

                return_data[0] = 0;
                for i in 1..return_data.len() {

                    return_data[i] -= return_data[i - 1];

                }

            },
            2 => {
                if data.len() < 3 {
                    return None;
                }
                //return_data[0] = 0;
                //return_data[1] = 0;
                for i in 2..return_data.len() {

                    let r = 2 * return_data[i - 1] - return_data[i - 2];
                    return_data[i] -= r;
                }

            },
            3 => {
                if data.len() < 4 {
                    return None;
                }

                //return_data[0] = 0;
                //return_data[1] = 0;
                //return_data[2] = 0;
                for i in 3..return_data.len() {

                    let r = 3 * return_data[i - 1] - 3 * data[i - 2] + data[i - 3];
                    return_data[i] -= r;
                }

            },
            4 => {
                if data.len() < 5 {
                    return None;
                }
                //return_data[0] = 0;
                //return_data[2] = 0;
                //return_data[3] = 0;
                //return_data[4] = 0;
                for i in 4..return_data.len() {

                    let r = 4 * data[i - 1] - 6 * data[i - 2] + 4 * data[i - 3] - data[i - 4];
                    return_data[i] -= r;
                }

            }

            _ => {
                return None;
            }

        }

        return Some(return_data);

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_ietf_02a() {
        let in_vec = vec![
            4302, 7496, 6199, 7427,
            6484, 7436, 6740, 7508,
            6984, 7583, 7182, -5990,
            -6306, -6032, -6299, -6165,
        ];

        let out_vec_ans = vec![
            3194, -1297, 1228,
            -943, 952, -696, 768,
            -524, 599, -401, -13172,
            -316, 274, -267, 134,
        ];

        let ans = FixedPredictor::get_residuals(&in_vec, 1);

        assert!(ans.is_some());
        assert_eq!(ans.unwrap(), out_vec_ans);
    }
}

fn main() {

}
