use ark_ff::{ BigInteger, PrimeField };
use multilinear::multilinear_poly::mult_polynomial::MultilinearPoly;
use multilinear::composed_poly::product_poly::ProductPoly;
use crate::circuit::circuit::{ Layer, Circuit };
use multilinear::composed_poly::sum_poly::SumPoly;
use sumcheck_protocol::sumcheck_protocol::transcript::Transcript;
use sumcheck_protocol::sumcheck_protocol::gkr_sumcheck::sumcheck::{
    gkr_partial_prove,
    gkr_partial_verify,
    GkrSumcheckProof,
};

#[derive(Debug)]
pub struct Proof<F: PrimeField> {
    pub circuit_output: Vec<F>,
    pub claimed_output: F,
    pub sumcheck_proofs: Vec<GkrSumcheckProof<F>>,
    pub wb_evaluation: Vec<F>,
    pub wc_evaluation: Vec<F>,
}

pub fn gkr_prover<F: PrimeField>(circuit: &mut Circuit<F>, input: Vec<F>) -> Proof<F> {
    let circuit_evaluation = circuit.evaluate(input);

    let mut transcript = Transcript::new();
    let mut layers = circuit.layers.clone();
    let mut proof_polys = Vec::new();
    let mut wb_evaluation = Vec::new();
    let mut wc_evaluation = Vec::new();
    let mut alpha = F::zero();
    let mut beta = F::zero();
    let mut current_rb = Vec::new();
    let mut current_rc = Vec::new();

    let output_poly = circuit.clone().get_w_i_poly(0);

    transcript.append(&convert_poly_to_bytes(output_poly.clone()));

    let random_challenge = transcript.sample_field_element();

    let mut claimed_sum = output_poly.evaluate_poly(&[random_challenge]).unwrap();

    transcript.append(claimed_sum.into_bigint().to_bytes_be().as_slice());

    layers.reverse();

    for (i, layer) in layers.into_iter().enumerate() {
        let fbc_poly = if i == 0 {
            f_b_c_poly(i, layer, circuit, random_challenge)
        } else {
            get_merged_fbc_poly(i, layer, circuit, &current_rb, &current_rc, alpha, beta)
        };

        let wb_poly = circuit.clone().get_w_i_poly(i + 1);
        let wc_poly = wb_poly.clone();

        let sum_check_proof = gkr_partial_prove(fbc_poly, claimed_sum, &mut transcript);
        proof_polys.push(sum_check_proof.clone());

        if i < circuit.layers.len() - 1 {
            let sumcheck_challenges = sum_check_proof.random_challenges;
            let (wb_evals, wc_evals) = evaluate_wb_wc(&wb_poly, &wc_poly, &sumcheck_challenges);

            wb_evaluation.push(wb_evals);
            wc_evaluation.push(wc_evals);

            let middle = sumcheck_challenges.len() / 2;
            let (current_rb_values, current_rc_values) = sumcheck_challenges.split_at(middle);
            current_rb = current_rb_values.to_vec();
            current_rc = current_rc_values.to_vec();

            transcript.append(wb_evals.into_bigint().to_bytes_be().as_slice());
            alpha = transcript.sample_field_element();

            transcript.append(wc_evals.into_bigint().to_bytes_be().as_slice());
            beta = transcript.sample_field_element();

            // Compute claimed sum using linear combination form
            claimed_sum = alpha * wb_evals + beta * wc_evals;
        }
    }

    Proof {
        circuit_output: circuit_evaluation,
        sumcheck_proofs: proof_polys,
        claimed_output: claimed_sum,
        wb_evaluation,
        wc_evaluation,
    }
}

pub fn verify<F: PrimeField>(
    circuit: &mut Circuit<F>,
    layer: &Layer,
    proof: Proof<F>,
    inputs: &[F]
) -> bool {
    let mut transcript = Transcript::new();
    let mut alpha = F::zero();
    let mut beta = F::zero();
    let mut prev_sumcheck_challenges = Vec::new();

    // layer 0 computation
    let w0_polynomial = if proof.circuit_output.len() == 1 {
        let mut w0_padded_with_zero = proof.circuit_output;
        w0_padded_with_zero.push(F::zero());
        MultilinearPoly::new(
            w0_padded_with_zero.clone(),
            w0_padded_with_zero.len().ilog2() as usize
        ).unwrap()
    } else {
        MultilinearPoly::new(
            proof.circuit_output.clone(),
            proof.circuit_output.len().ilog2() as usize
        ).unwrap()
    };

    transcript.append(
        w0_polynomial.evaluated_value
            .iter()
            .flat_map(|f| f.into_bigint().to_bytes_be())
            .collect::<Vec<_>>()
            .as_slice()
    );
    let random_challenge_a: F = transcript.sample_field_element();

    let mut claimed_sum = w0_polynomial.evaluate_poly(&vec![random_challenge_a]).unwrap();

    for layer_index in 0..circuit.layers.len() {
        if claimed_sum != proof.sumcheck_proofs[layer_index].claimed_sum {
            return false;
        }

        // Get the verification result
        let verify_result = gkr_partial_verify(
            proof.sumcheck_proofs[layer_index].clone(),
            &mut transcript
        );
        if !verify_result.valid_proof {
            return false;
        }

        let sumcheck_challenges = verify_result.random_challenges;

        let (wb_evaluation, wc_evaluation) = if layer_index < circuit.layers.len() - 1 {
            (proof.wb_evaluation[layer_index], proof.wc_evaluation[layer_index])
        } else {
            let wb_poly = MultilinearPoly::new(
                inputs.to_vec(),
                inputs.len().ilog2() as usize
            ).unwrap();
            let wc_poly = wb_poly.clone();

            evaluate_wb_wc(&wb_poly, &wc_poly, &sumcheck_challenges)
        };

        let expected_claim = if layer_index == 0 {
            get_verifier_claim(
                layer.clone(),
                layer_index,
                random_challenge_a,
                &sumcheck_challenges,
                wb_evaluation,
                wc_evaluation
            )
        } else {
            get_folded_verifier_claim(
                layer.clone(),
                layer_index,
                &sumcheck_challenges,
                &prev_sumcheck_challenges,
                wb_evaluation,
                wc_evaluation,
                alpha,
                beta
            )
        };

        if expected_claim != verify_result.final_claimed_sum {
            return false;
        }

        prev_sumcheck_challenges = sumcheck_challenges.to_vec();

        transcript.append(wb_evaluation.into_bigint().to_bytes_be().as_slice());
        alpha = transcript.sample_field_element();

        transcript.append(wc_evaluation.into_bigint().to_bytes_be().as_slice());
        beta = transcript.sample_field_element();

        claimed_sum = alpha * wb_evaluation + beta * wc_evaluation;
    }

    true
}

pub fn f_b_c_poly<F: PrimeField>(
    layer_index: usize,
    layer: Layer,
    circuit: &Circuit<F>,
    randomchallenge: F
) -> SumPoly<F> {
    let summed_w_poly = circuit.clone().add_w_b_c_poly(layer_index);
    let multiplied_w_poly = circuit.clone().mul_w_b_c_poly(layer_index);

    let add_i = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .0.partial_evaluate(0, randomchallenge);

    let mul_i = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .1.partial_evaluate(0, randomchallenge);

    let add_eval_product = ProductPoly::new(vec![add_i, summed_w_poly.clone()]);
    let mul_eval_product = ProductPoly::new(vec![mul_i, multiplied_w_poly.clone()]);

    let f_b_c = SumPoly::new(vec![add_eval_product, mul_eval_product]);

    f_b_c
}

fn get_merged_fbc_poly<F: PrimeField>(
    layer_index: usize,
    layer: Layer,
    circuit: &Circuit<F>,
    r_b: &[F],
    r_c: &[F],
    alpha: F,
    beta: F
) -> SumPoly<F> {
    let summed_w_poly = circuit.clone().add_w_b_c_poly(layer_index);
    let multiplied_w_poly = circuit.clone().mul_w_b_c_poly(layer_index);

    let add_i_r_b = layer.clone().get_add_i_and_mul_i(layer_index).0.partial_evaluate(0, r_b[0]);

    let add_i_r_c = layer.clone().get_add_i_and_mul_i(layer_index).0.partial_evaluate(0, r_c[0]);

    let mul_i_r_b = layer.clone().get_add_i_and_mul_i(layer_index).1.partial_evaluate(0, r_b[0]);

    let mul_i_r_c = layer.clone().get_add_i_and_mul_i(layer_index).1.partial_evaluate(0, r_c[0]);

    let new_add_i = add_i_r_b.scale(alpha) + add_i_r_c.scale(beta);
    let new_mul_i = mul_i_r_b.scale(alpha) + mul_i_r_c.scale(beta);

    let add_eval_product = ProductPoly::new(vec![new_add_i, summed_w_poly.clone()]);
    let mul_eval_product = ProductPoly::new(vec![new_mul_i, multiplied_w_poly.clone()]);

    let merged_f_b_c = SumPoly::new(vec![add_eval_product, mul_eval_product]);

    merged_f_b_c
}

pub fn get_verifier_claim<F: PrimeField>(
    layer: Layer,
    layer_index: usize,
    init_random_challenge: F,
    sumcheck_random_challenges: &[F],
    wb_evaluation: F,
    wc_evaluation: F
) -> F {
    let mut all_random_challenges = Vec::with_capacity(1 + sumcheck_random_challenges.len());

    all_random_challenges.push(init_random_challenge);
    all_random_challenges.extend_from_slice(sumcheck_random_challenges);

    let a_r = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .0.evaluate_poly(&all_random_challenges)
        .unwrap();
    let m_r = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .1.evaluate_poly(&all_random_challenges)
        .unwrap();

    a_r * (wb_evaluation + wc_evaluation) + m_r * (wb_evaluation * wc_evaluation)
}

fn get_folded_verifier_claim<F: PrimeField>(
    layer: Layer,
    layer_index: usize,
    current_random_challenge: &[F],
    previous_random_challenge: &[F],
    wb_evaluation: F,
    wc_evaluation: F,
    alpha: F,
    beta: F
) -> F {
    let (prev_r_b, prev_r_c) = previous_random_challenge.split_at(
        previous_random_challenge.len() / 2
    );

    let add_i_r_b = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .0.partial_evaluate(0, prev_r_b[0]);

    let add_i_r_c = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .0.partial_evaluate(0, prev_r_c[0]);

    let mul_i_r_b = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .1.partial_evaluate(0, prev_r_b[0]);

    let mul_i_r_c = layer
        .clone()
        .get_add_i_and_mul_i(layer_index)
        .1.partial_evaluate(0, prev_r_c[0]);

    let new_add_i = add_i_r_b.scale(alpha) + add_i_r_c.scale(beta);
    let new_mul_i = mul_i_r_b.scale(alpha) + mul_i_r_c.scale(beta);

    let add_r = new_add_i.evaluate_poly(&current_random_challenge.to_vec()).unwrap();
    let mul_r = new_mul_i.evaluate_poly(&current_random_challenge.to_vec()).unwrap();

    add_r * (wb_evaluation + wc_evaluation) + mul_r * (wb_evaluation * wc_evaluation)
}

pub fn tensor_addition<F: PrimeField>(
    b: &MultilinearPoly<F>,
    c: &MultilinearPoly<F>
) -> MultilinearPoly<F> {
    let result: Vec<F> = b.evaluated_value
        .iter()
        .flat_map(|&x| c.evaluated_value.iter().map(move |&y| x + y))
        .collect();
    // dbg!(&result);
    MultilinearPoly::new(result.clone(), result.len().ilog2() as usize).unwrap()
}

pub fn tensor_multiplication<F: PrimeField>(
    b: &MultilinearPoly<F>,
    c: &MultilinearPoly<F>
) -> MultilinearPoly<F> {
    let result: Vec<F> = b.evaluated_value
        .iter()
        .flat_map(|&x| c.evaluated_value.iter().map(move |&y| x * y))
        .collect();
    MultilinearPoly::new(result.clone(), result.len().ilog2() as usize).unwrap()
}

pub fn evaluate_wb_wc<F: PrimeField>(
    wb_poly: &MultilinearPoly<F>,
    wc_poly: &MultilinearPoly<F>,
    sumcheck_challenges: &[F]
) -> (F, F) {
    let middle = sumcheck_challenges.len() / 2;
    let (rb_values, rc_values) = sumcheck_challenges.split_at(middle);

    let wb_poly_evaluated = wb_poly.clone().evaluate_poly(rb_values).unwrap();
    let wc_poly_evaluated = wc_poly.clone().evaluate_poly(rc_values).unwrap();

    (wb_poly_evaluated, wc_poly_evaluated)
}

fn convert_poly_to_bytes<F: PrimeField>(poly: MultilinearPoly<F>) -> Vec<u8> {
    poly.evaluated_value
        .iter()
        .flat_map(|f| f.into_bigint().to_bytes_be())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;
    use crate::circuit::circuit::{ Gate, Opt };

    #[test]
    fn test_f_b_c_poly() {
        let layer1 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add),
                Gate::new(1, 2, 3, Opt::Mul),
                Gate::new(2, 4, 5, Opt::Mul),
                Gate::new(3, 6, 7, Opt::Mul)
            ]
        );
        let layer2 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add), Gate::new(1, 2, 3, Opt::Mul)]);
        let layer3 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add)]);
        let mut circuit = Circuit::new(vec![layer1.clone(), layer2.clone(), layer3.clone()]);

        circuit.evaluate(
            vec![
                Fq::from(1),
                Fq::from(2),
                Fq::from(3),
                Fq::from(4),
                Fq::from(5),
                Fq::from(6),
                Fq::from(7),
                Fq::from(8)
            ]
        );

        let add = f_b_c_poly(0, layer3, &circuit, Fq::from(0));
        dbg!(add);
    }

    #[test]
    fn test_merged_f_b_c_poly() {
        let layer1 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add),
                Gate::new(1, 2, 3, Opt::Mul),
                Gate::new(2, 4, 5, Opt::Mul),
                Gate::new(3, 6, 7, Opt::Mul)
            ]
        );
        let layer2 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add), Gate::new(1, 2, 3, Opt::Mul)]);
        let layer3 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add)]);
        let mut circuit = Circuit::new(vec![layer1.clone(), layer2.clone(), layer3.clone()]);

        circuit.evaluate(
            vec![
                Fq::from(1),
                Fq::from(2),
                Fq::from(3),
                Fq::from(4),
                Fq::from(5),
                Fq::from(6),
                Fq::from(7),
                Fq::from(8)
            ]
        );

        let add = get_merged_fbc_poly(
            1,
            layer2,
            &circuit,
            &[Fq::from(0)],
            &[Fq::from(0)],
            Fq::from(2),
            Fq::from(2)
        );

        dbg!(add);
    }

    #[test]
    fn test_tensor_addition() {
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let c = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let d = vec![
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(5),
            Fq::from(3),
            Fq::from(4),
            Fq::from(5),
            Fq::from(6),
            Fq::from(4),
            Fq::from(5),
            Fq::from(6),
            Fq::from(7),
            Fq::from(5),
            Fq::from(6),
            Fq::from(7),
            Fq::from(8)
        ];
        let poly_1 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(c.clone(), c.len().ilog2() as usize).unwrap();
        let result = MultilinearPoly::new(d.clone(), d.len().ilog2() as usize).unwrap();
        assert_eq!(tensor_addition(&poly_1, &poly_2), result);
    }

    #[test]
    fn test_tensor_multiplication() {
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let c = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let d = vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(2),
            Fq::from(4),
            Fq::from(6),
            Fq::from(8),
            Fq::from(3),
            Fq::from(6),
            Fq::from(9),
            Fq::from(12),
            Fq::from(4),
            Fq::from(8),
            Fq::from(12),
            Fq::from(16)
        ];
        let poly_1 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(c.clone(), c.len().ilog2() as usize).unwrap();
        let result = MultilinearPoly::new(d.clone(), d.len().ilog2() as usize).unwrap();
        assert_eq!(tensor_multiplication(&poly_1, &poly_2), result);
    }

    #[test]
    fn test_gkr_prove() {
        let layer1 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add),
                Gate::new(1, 2, 3, Opt::Mul),
                Gate::new(2, 4, 5, Opt::Mul),
                Gate::new(3, 6, 7, Opt::Mul)
            ]
        );
        let layer2 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add), Gate::new(1, 2, 3, Opt::Mul)]);
        let layer3 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add)]);
        let mut circuit = Circuit::new(vec![layer1.clone(), layer2.clone(), layer3.clone()]);

        let input = vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(5),
            Fq::from(6),
            Fq::from(7),
            Fq::from(8)
        ];

        let proof = gkr_prover(&mut circuit, input.clone());
        dbg!(&proof);

        // let verify = verify(&mut circuit, &layer1, proof, &input);
        // dbg!(&verify);
    }
}
