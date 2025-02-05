use crate::multilinear_poly::mult_polynomial::MultilinearPoly;
use crate::sumcheck_protocol::transcript::Transcript;
use ark_ff::{BigInteger, PrimeField};

#[derive(Debug)]
struct Proof<F: PrimeField> {
    claimed_sum: F,
    round_polys: Vec<[F; 2]>,
}

fn prove<F: PrimeField>(poly: &MultilinearPoly<F>, claimed_sum: F) -> Proof<F> {
    let mut round_polys = vec![];

    let mut transcript = Transcript::new();
    // &[u8]
    // [&[u8], &[u8], ...]
    // [[1, 2], [3, 4]]
    // [1, 2, 3, 4]
    transcript.append(
        poly.evaluated_value
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    transcript.append(claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut poly = poly.clone();


    for _ in 0..poly.num_vars {
        let round_poly: [F; 2] = [
            poly.partial_evaluate(0, F::from(1)).evaluated_value.iter().sum(),
            poly.partial_evaluate(0, F::zero()).evaluated_value.iter().sum(),
        ];

        transcript.append(
            round_poly
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        round_polys.push(round_poly);

        let challenge = transcript.sample_field_element();

        poly = poly.partial_evaluate(0,  challenge);
     
    }

    Proof {
        claimed_sum,
        round_polys,
    }
}

fn verify<F: PrimeField>(poly: &MultilinearPoly<F>, proof: &Proof<F>) -> bool {
    if proof.round_polys.len() != poly.num_vars {
        return false;
    }

    let mut challenges = vec![];

    let mut transcript = Transcript::new();
    transcript.append(
        poly.evaluated_value
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    transcript.append(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut claimed_sum = proof.claimed_sum;

    for round_poly in &proof.round_polys {
        if claimed_sum != round_poly.iter().sum() {
            return false;
        }

        transcript.append(
            round_poly
                .iter()
                .flat_map(|f| f.into_bigint().to_bytes_be())
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let challenge = transcript.sample_field_element();
        claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
        challenges.push(challenge);
    }

    if claimed_sum != poly.clone().evaluate_poly(&challenges).expect("error message") {
        return false;
    }

    true
}

#[cfg(test)]
mod test {

    use crate::multilinear_poly::mult_polynomial::MultilinearPoly;

    use crate::sumcheck_protocol::sumcheck::{ prove, verify };
    use ark_bn254::Fq as ArkField;

    use field_tracker::{ end_tscope, Ft, print_summary, start_tscope };

    type Fq = Ft!(ArkField);

    #[test]
    fn test_sumcheck() {
        start_tscope!("sumcheck");
        let poly = MultilinearPoly::new(
            vec![
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(3),
                Fq::from(2),
                Fq::from(5)
            ],
            3
        ).expect("expecte");
        let proof = prove(&poly, Fq::from(20));
        dbg!(&proof);
        end_tscope!();
        print_summary!();
        dbg!(verify(&poly, &proof));
    }
}