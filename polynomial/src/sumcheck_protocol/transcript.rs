use ark_ff::{BigInteger, PrimeField};
use sha3::{Digest, Keccak256};
use std::marker::PhantomData;

// Transcript for Fiat-Shamir heuristic
pub struct Transcript<K: HashTrait, F: PrimeField> {
    _field: PhantomData<F>,
    hash_function: K,
}

impl<K: HashTrait, F: PrimeField> Transcript<K, F> {
    /// Initialize new transcript with specified hash function
    pub fn new(hash_function: K) -> Self {
        Self {
            _field: PhantomData,
            hash_function,
        }
    }

    /// Absorb data into the transcript (add to hash state)
    pub fn absorb(&mut self, data: &[u8]) {
        self.hash_function.append(data);
    }

    /// Generate field element challenge from current hash state
    pub fn squeeze_challenge(&mut self) -> F {
        let hash_output = self.hash_function.generate_hash();
        F::from_be_bytes_mod_order(&hash_output)
    }

    /// Squeeze raw bytes challenge (for non-field elements)
    pub fn squeeze_bytes(&mut self, output_len: usize) -> Vec<u8> {
        let mut hash = self.hash_function.generate_hash();
        hash.truncate(output_len);
        hash
    }
}

/// Trait for hash function compatibility
pub trait HashTrait: Clone {
    fn append(&mut self, data: &[u8]);
    fn generate_hash(&self) -> Vec<u8>;
}

impl HashTrait for Keccak256 {
    fn append(&mut self, data: &[u8]) {
        Digest::update(self, data);
    }

    fn generate_hash(&self) -> Vec<u8> {
        self.clone().finalize().to_vec()
    }
}

/// Multilinear Polynomial implementation
pub struct MultilinearPoly<F: PrimeField> {
    pub evaluations: Vec<F>,
    num_vars: usize,
}

impl<F: PrimeField> MultilinearPoly<F> {
    /// Create new multilinear polynomial from evaluations
    pub fn new(evaluations: Vec<F>) -> Self {
        let num_vars = evaluations.len().ilog2() as usize;
        assert_eq!(
            evaluations.len(),
            1 << num_vars,
            "Evaluation length must be power of two"
        );
        
        Self {
            evaluations,
            num_vars,
        }
    }

    /// Perform partial evaluation by fixing one variable
    pub fn partial_evaluate(&self, fixed_var_index: usize, fixed_value: F) -> Self {
        assert!(
            fixed_var_index < self.num_vars,
            "Variable index out of bounds"
        );

        let mut new_evals = Vec::with_capacity(self.evaluations.len() / 2);
        
        for i in 0..self.evaluations.len() / 2 {
            let x0 = self.evaluations[i];
            let x1 = self.evaluations[i + (1 << (self.num_vars - 1 - fixed_var_index))];
            new_evals.push(x0 + fixed_value * (x1 - x0));
        }

        MultilinearPoly {
            evaluations: new_evals,
            num_vars: self.num_vars - 1,
        }
    }

    /// Full evaluation at all points (already in evaluation form)
    pub fn full_evaluate(&self) -> &[F] {
        &self.evaluations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::One;

    #[test]
    fn test_transcript() {
        let mut transcript = Transcript::<Keccak256, Fr>::new(Keccak256::new());
        
        // Test basic absorption
        transcript.absorb(b"test_data");
        let challenge = transcript.squeeze_challenge();
        assert_ne!(challenge, Fr::from(0));
    }

    #[test]
    fn test_partial_evaluation() {
        // Coefficients for 3-variable polynomial: 2x0x1 + x2
        let coefficients = vec![
            Fr::from(0), Fr::from(1),  // x2=0
            Fr::from(2), Fr::from(3),  // x2=1
        ];
        
        let poly = MultilinearPoly::new(coefficients);
        let fixed_var_index = 0;  // Fix x0
        let fixed_value = Fr::one();
        
        let partial_poly = poly.partial_evaluate(fixed_var_index, fixed_value);
        
        // Expected results after fixing x0=1:
        // When x0=1: 2*1*x1 + x2
        let expected = vec![Fr::from(1), Fr::from(3)];  // x1=0 and x1=1
        
        assert_eq!(partial_poly.evaluations, expected);
    }

    #[test]
    fn test_full_evaluation() {
        // Simple 2-variable polynomial: x0 + x1
        let coefficients = vec![
            Fr::from(0),  // 00
            Fr::from(1),  // 01
            Fr::from(1),  // 10
            Fr::from(2),  // 11
        ];
        
        let poly = MultilinearPoly::new(coefficients);
        let evaluations = poly.full_evaluate();
        
        assert_eq!(evaluations.len(), 4);
        assert_eq!(evaluations[3], Fr::from(2));
    }
}