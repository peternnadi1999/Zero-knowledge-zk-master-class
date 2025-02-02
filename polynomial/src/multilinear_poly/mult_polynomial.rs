use ark_ff::PrimeField;

#[derive(Clone)]
pub struct MultilinearPoly<F: PrimeField> {
    pub evaluated_value: Vec<F>,
    num_vars: u32,
}

impl<F: PrimeField> MultilinearPoly<F> {
    pub fn new(evaluate: Vec<F>, num_vars: u32) -> Result<Self, &'static str> {
        if num_vars == 0 {
            return Err("Number of variables must be greater than 0");
        }
        if evaluate.len() != (1 << num_vars) {
            return Err("The length of the evaluated value must be 2^num_vars");
        }
        // Check if the evaluated value is valid
        Ok(Self {
            evaluated_value: evaluate,
            num_vars,
        })
    }

    fn partial_evaluate(
        &self,
        fixed_var_value: F, // the value of a,b,c. a=4, b=1, c=2
        fixed_var_index: u32 //0,1,2 for 3 variables and  0,1 for 2 variables
    ) -> Vec<F> {
        // Ensure fixed_var_index is valid
        if fixed_var_index >= self.num_vars {
            panic!("fixed_var_index is out of bounds");
        }
        let subset_size = 1 << (self.num_vars - 1);
        let mut result = vec![F::zero(); subset_size]; // Initialize result vector with zeros

        // when you are looking for a
        // 00 - (000, 100) - (0, 4)
        // 01 - (001, 101) - (1, 5)
        // 10 - (010, 110) - (2, 6)
        // 11 - (011, 111) - (3, 7)
        // when you are looking for b
        // 00 - (000, 010) - (0, 2)
        // 01 - (001, 011) - (1, 3)
        // 10 - (100, 110) - (4, 6)
        // 11 - (101, 111) - (5, 7)
        // when you are looking for c
        // 00 - (000, 001) - (0, 1)
        // 01 - (010, 011) - (2, 3)
        // 10 - (100, 101) - (4, 5)
        // 11 - (110, 111) - (6, 7)

        let bit_pos = self.num_vars - fixed_var_index - 1;

        for i in 0..subset_size {
            let mask = (1 << bit_pos) - 1;
            let original_index_0 = (i & mask) | ((i >> bit_pos) << (bit_pos + 1));
            let original_index_1 = original_index_0 | (1 << bit_pos);

            let y1 = self.evaluated_value[original_index_0];
            let y2 = self.evaluated_value[original_index_1];
            result[i as usize] = y1 + fixed_var_value * (y2 - y1);
        }
        result
    }
    

    pub fn evaluate_poly(self, assignment: &[F]) -> Result<F, &str> {
        if assignment.len() != (self.num_vars as usize) {
            return Err("Assignment length does not match the number of variables");
        }

        let mut poly = self.clone();

        for i in assignment{
            poly.evaluated_value = poly.partial_evaluate(*i, 0 );
            poly.num_vars -= 1;
        }
        Ok(poly.evaluated_value[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_partial_evaluate() {
        let evaluated_value = vec![
            Fq::from(0),
            Fq::from(3),
            Fq::from(0),
            Fq::from(3),
            Fq::from(0),
            Fq::from(3),
            Fq::from(2),
            Fq::from(5)
        ];
        let fixed_var_index = 0;
        let fixed_var_value = Fq::from(1_u32);
        let polynomial = MultilinearPoly::new(evaluated_value.clone(), 3).expect(
            "Failed to create MultilinearPoly"
        );
        let result = polynomial.partial_evaluate(fixed_var_value, fixed_var_index);
        assert_eq!(result, vec![Fq::from(0), Fq::from(3), Fq::from(2), Fq::from(5)]);
    }

    #[test]
    fn test_evaluation() {
        let evaluated_value = vec![
            Fq::from(0),
            Fq::from(3),
            Fq::from(0),
            Fq::from(3),
            Fq::from(0),
            Fq::from(3),
            Fq::from(2),
            Fq::from(5)
        ];
        let polynomial = MultilinearPoly::new(evaluated_value.clone(), 3).expect(
            "Failed to create MultilinearPoly"
        );
        let assignment = vec![Fq::from(1_u32), Fq::from(1_u32), Fq::from(1_u32)];
        let result = polynomial.evaluate_poly(&assignment).expect("Evaluation failed");
        assert_eq!(result, Fq::from(5)); // Expected result based on the polynomial and assignment
    }
}
