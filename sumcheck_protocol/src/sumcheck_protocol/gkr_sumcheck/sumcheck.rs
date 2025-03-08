use multilinear::composed_poly::sum_poly::SumPoly;
use univariate_polynomial::univariant_poly::polynomial::UnivariatePoly;
use ark_ff::{ BigInteger, PrimeField };
use crate::sumcheck_protocol::transcript::Transcript;

#[derive(Debug, Clone)]
pub struct GkrSumcheckProof<F: PrimeField> {
    pub proof_poly: Vec<UnivariatePoly<F>>,
    pub claimed_sum: F,
    pub random_challenges: Vec<F>,
}

#[derive(Clone, Debug)]
pub struct GKRSumcheckVerify<F: PrimeField> {
    pub valid_proof: bool,
    pub random_challenges: Vec<F>,
    pub final_claimed_sum: F,
}


pub fn gkr_partial_prove<F: PrimeField>(
    sum_polynomial: SumPoly<F>,
    claimed_sum: F,
    transcript: &mut Transcript
) -> GkrSumcheckProof<F> {
    let num_var = sum_polynomial.poly[0].poly[0].num_vars;
    
    let mut poly = sum_polynomial.clone();
    let mut random_challenges = Vec::with_capacity(num_var);
    let mut proof_poly = Vec::new();
    
    transcript.append(&convert_claims_to_bytes(claimed_sum));

    for _ in 0..num_var {
        let degree = poly.degree();
        let mut results = Vec::with_capacity(degree + 1);
        
        for i in 0..degree + 1{
            let partial_poly = poly.partial_evaluate(0, F::from(i as u64));

            let sum: F = partial_poly.reduce().iter().sum();
            results.push(sum);
           
        }
        let x: Vec<F> = (0..=sum_polynomial.degree()).map(|i| F::from(i as u64)).collect();
     
        let univariate_poly = UnivariatePoly::interpolate(x, results);
           
        transcript.append(&convert_poly_to_bytes(&univariate_poly.coefficient));

        proof_poly.push(univariate_poly);
      
        let challenge = transcript.sample_field_element();

        poly = poly.partial_evaluate(0, challenge);
        
        random_challenges.push(challenge);
     
        
    }

    GkrSumcheckProof {
        proof_poly,
        claimed_sum,
        random_challenges,
    }
}

pub fn gkr_partial_verify<F: PrimeField>(
     proof: GkrSumcheckProof<F>,
    transcript: &mut Transcript,
) -> GKRSumcheckVerify<F> {
    let mut random_challenges = Vec::with_capacity(proof.proof_poly.len());
    let mut claimed_sum = proof.claimed_sum;

    transcript.append(&convert_claims_to_bytes(proof.claimed_sum));
    dbg!(&proof.proof_poly);

    for round_poly in proof.proof_poly {
        //evaluate the round poly at 0 and 1
        let round_poly_eval_0 = round_poly.evaluate(F::zero());
        let round_poly_eval_1 = round_poly.evaluate(F::one());

        dbg!(round_poly_eval_0);
        dbg!(round_poly_eval_1);
        dbg!(claimed_sum);

        if round_poly_eval_0 + round_poly_eval_1 != claimed_sum {
            return GKRSumcheckVerify {
                valid_proof: false,
                final_claimed_sum: F::zero(),
                random_challenges: vec![F::zero()],
            };
        }

        transcript.append(&convert_poly_to_bytes(&round_poly.coefficient));

        let challenge = transcript.sample_field_element();
        
        random_challenges.push(challenge);

        claimed_sum = round_poly.evaluate(challenge); //next expected sum

        dbg!(&round_poly.coefficient);
        dbg!(challenge);
        dbg!(round_poly.evaluate(challenge));
            
    }

    GKRSumcheckVerify {
        valid_proof: true,
        random_challenges,
        final_claimed_sum: claimed_sum,
    }
}

fn convert_poly_to_bytes<F: PrimeField>(poly: &[F]) -> Vec<u8> {
    poly.iter()
        .flat_map(|f| f.into_bigint().to_bytes_be())
        .collect::<Vec<_>>()
}

fn convert_claims_to_bytes<F: PrimeField>(claims: F) -> Vec<u8> {
    claims.into_bigint().to_bytes_be().as_slice().to_vec()
}

#[cfg(test)]
mod test {
    use multilinear::{composed_poly::{product_poly::ProductPoly, sum_poly::SumPoly}, multilinear_poly::mult_polynomial::MultilinearPoly};

    use crate::sumcheck_protocol::gkr_sumcheck::sumcheck::{gkr_partial_verify, gkr_partial_prove };
    use ark_bn254::Fq ;
    use crate::sumcheck_protocol::transcript::Transcript;

    #[test]
    fn test_gkr_prove_verify() {
        let a = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)];
        let b = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)];

        let c = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)];
        let d = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)];

        let poly_1 = MultilinearPoly::new(a.clone(), a.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let poly_3 = MultilinearPoly::new(c.clone(), b.len().ilog2() as usize).unwrap();
        let poly_4 = MultilinearPoly::new(d.clone(), c.len().ilog2() as usize).unwrap();

        let product_poly1 = ProductPoly::new(vec![poly_1.clone(), poly_2.clone()]);
        let product_poly2 = ProductPoly::new(vec![poly_3.clone(), poly_4.clone()]);

        let fbc_poly = SumPoly::new(vec![product_poly1, product_poly2]);

        let mut prove_transcript = Transcript::new();
        let mut verify_transcript = Transcript::new();

        let result = gkr_partial_prove(fbc_poly, Fq::from(12),  &mut prove_transcript);
        
        dbg!(&result);

        let verified = gkr_partial_verify(
            result,
            &mut verify_transcript,
        );

        assert_eq!(verified.valid_proof, true);

    }

}

