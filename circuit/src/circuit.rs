use ark_ff::PrimeField;
use multilinear::multilinear_poly::mult_polynomial::MultilinearPoly;

#[derive(Debug, PartialEq)]
struct Gate {
    output: usize,
    left_index: usize, // Index for the left input
    right_index: usize, // Index for the right input
    opt: Opt,
}

#[derive(Debug, PartialEq)]
enum Opt {
    Add,
    Mul,
}

#[derive(Debug, PartialEq, Default)]
struct Layer {
    gates: Vec<Gate>,
}

#[derive(Debug, PartialEq)]
struct Circuit<F: PrimeField> {
    layers: Vec<Layer>,
    layer_evals: Vec<Vec<F>>,
}

impl Gate {
    fn new(output: usize, left_index: usize, right_index: usize, opt: Opt) -> Self {
        Self {
            output: output, // Placeholder, will be updated after evaluation
            left_index,
            right_index,
            opt,
        }
    }

    fn evaluate<F: PrimeField>(&mut self, inputs: &Vec<F>, outputs: &mut Vec<F>) {
        let left_input = inputs[self.left_index];
        let right_input = inputs[self.right_index];

        let output = match self.opt {
            Opt::Add => left_input + right_input,
            Opt::Mul => left_input * right_input,
        };
        outputs[self.output] += output;
    }
}

impl Layer {
    fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }

    fn evaluate<F: PrimeField>(&mut self, inputs: &Vec<F>) -> Vec<F> {
        let mut outputs = vec![F::zero(); self.gates.len()];
        for gate in &mut self.gates {
            gate.evaluate(inputs, &mut outputs);
        }

        outputs
    }

    fn get_add_i_and_mul_i<F: PrimeField>(
        self,
        layer_index: usize
    ) -> (MultilinearPoly<F>, MultilinearPoly<F>) {
        let num_vars = self.get_num_var(layer_index);
        let num_points = 1 << num_vars; // 2^num_vars

        let mut add_i = vec![F::zero(); num_points];
        let mut mul_i = vec![F::zero(); num_points];
        dbg!(num_vars);
        let width = layer_index + 1;
        for (i, gate) in self.gates.iter().enumerate() {
            let output = format!("{:0width$b}", gate.output, width = layer_index);
            let left_binary = format!("{:0width$b}", gate.left_index, width = width);
            let right_binary = format!("{:0width$b}", gate.right_index, width = width);
            dbg!(&output);
            // Concatenate the binary strings
            let concatenated = format!("{}{}{}", output, left_binary, right_binary);
            let index = usize::from_str_radix(concatenated.as_str(), 2).unwrap();
            match gate.opt {
                Opt::Add => {
                    add_i[index] = F::one();
                }
                Opt::Mul => {
                    mul_i[index] = F::one();
                }
            }

            dbg!(index);
            dbg!(right_binary);
        }

        let add_poly = MultilinearPoly::new(add_i.clone(), add_i.len().ilog2() as usize).expect(
            "error"
        );
        let mul_poly = MultilinearPoly::new(add_i.clone(), add_i.len().ilog2() as usize).expect(
            "error"
        );

        (add_poly, mul_poly)
    }

    fn get_num_var(&self, layer_index: usize) -> usize {
        if layer_index == 0 {
            3
        } else {
            let variable_a = layer_index;
            let variable_b = layer_index + 1;
            let variable_c = layer_index + 1;
            variable_a + variable_b + variable_c
        }
    }
}

impl<F: PrimeField> Circuit<F> {
    fn new(layers: Vec<Layer>) -> Self {
        Self { layers, layer_evals: Vec::new() }
    }

    fn evaluate(&mut self, initial_inputs: Vec<F>) -> Vec<F> {
        let mut current_inputs = initial_inputs.clone();
        self.layer_evals.push(current_inputs.clone());

        for layer in &mut self.layers {
            current_inputs = layer.evaluate(&current_inputs);
            self.layer_evals.push(current_inputs.clone());
        }
        // println!("{:?}", self.layer_evals);
        self.layer_evals.reverse();
        current_inputs
    }

    fn get_w_i_poly(mut self, layer_index: usize) -> MultilinearPoly<F> {
        if layer_index < self.layer_evals.len() - 1 {
            panic!("Number of variables must not be greater than the number of inputs");
        }

        let mut num_var = self.layer_evals[layer_index].len().ilog2() as usize;

        if num_var == 0 {
            panic!("Number of variables must be greater than 0");
        }
        if self.layer_evals[layer_index].len() == 1 {
            self.layer_evals[layer_index].push(F::zero());
            num_var = self.layer_evals[layer_index].len().ilog2() as usize;
        }

        MultilinearPoly::new(self.layer_evals[layer_index].clone(), num_var).expect(
            "msg: failed to create multilinear polynomial"
        )
    }
}

#[cfg(test)]
mod tests {
    use std::process::Output;

    use ark_bn254::Fq;
    use super::*;

    #[test]
    fn test_gate(){
        let mut gate = Gate::new(0, 0, 1, Opt::Add);
        assert_eq!(gate.right_index, 1);
        assert_eq!(gate.output, 0);
        assert_eq!(gate.opt, Opt::Add);
        assert_eq!(gate.left_index, 0);
    }

    #[test]
    fn test_gate_evaluation() {
        let mut gate = Gate::new(0, 0, 1, Opt::Add);
       
        let inputs = &vec![Fq::from(2), Fq::from(3)];
        let mut output =vec![Fq::from(0); 2];
        gate.evaluate(inputs, &mut output);
        assert_eq!(output[0], Fq::from(5));
    }

    #[test]
    fn test_layer_evaluation() {
        let gates = vec![
            Gate::new(0, 0, 1, Opt::Add),
            Gate::new(1, 2, 3, Opt::Mul),
            Gate::new(2, 0, 1, Opt::Add) // Uses output of the first gate and an external input
        ];

        let mut layer = Layer::new(gates);
        let outputs = layer.evaluate(&vec![Fq::from(2), Fq::from(5), Fq::from(3), Fq::from(4)]);
        assert_eq!(outputs, vec![Fq::from(7), Fq::from(12), Fq::from(7)]);
    }

    #[test]
    fn test_circuit_evaluation() {
        let layer1 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add), // 2 + 3 = 5
                Gate::new(1, 2, 3, Opt::Mul), // 2 * 3 = 6
                Gate::new(2, 4, 5, Opt::Mul),
                Gate::new(3, 6, 7, Opt::Mul)
            ]
        );

        let layer2 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add), // 5 + 6 = 11
                Gate::new(1, 2, 3, Opt::Mul) // 2 * 3 = 6
            ]
        );
        let layer3 = Layer::new(vec![Gate::new(0, 0, 1, Opt::Add)]);

        let mut circuit = Circuit::new(vec![layer1, layer2, layer3]);

        let outputs = circuit.evaluate(
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

        assert_eq!(outputs, vec![Fq::from(1695)]);
    }

    #[test]
    fn test_get_w_i_poly() {
        let layer1 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add), // 2 + 3 = 5
                Gate::new(1, 2, 3, Opt::Mul), // 4 + 5 = 9
                Gate::new(2, 4, 5, Opt::Mul), // 6 * 7 = 42
                Gate::new(3, 6, 7, Opt::Mul) // 8 * 9 = 72
            ]
        );
        let layer2 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add), // 5 + 9 = 14
                Gate::new(1, 2, 3, Opt::Mul) // 42 + 72 = 114          114
            ]
        );
        let layer3 = Layer::new(
            vec![
                Gate::new(0, 0, 1, Opt::Add) // 14 + 114 = 128
            ]
        );
        let mut circuit = Circuit::new(vec![layer1, layer2, layer3]);

        let outputs = circuit.evaluate(
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
        let w_i_poly = circuit.get_w_i_poly(3);
        // dbg!(w_i_poly.evaluated_value);
        // assert_eq!(
        //     &w_i_poly.evaluated_value,
        //     vec![Fq::from(3), Fq::from(12), Fq::from(30), Fq::from(56)]
        // );
    }

    #[test]
    fn test_get_add_i_and_mul_i() {
        let gates = vec![
            Gate::new(0, 0, 1, Opt::Add), // Uses output of the first gate and an external input
            Gate::new(1, 2, 3, Opt::Add),
            // Gate::new(2, 4, 5, Opt::Add),
            // Gate::new(3, 6, 7, Opt::Mul),
            
        ];

        let mut layer = Layer::new(gates);
        layer.evaluate(&vec![Fq::from(1),
                Fq::from(2),
                Fq::from(3),
                Fq::from(4),
                Fq::from(5),
                Fq::from(6),
                Fq::from(7),
                Fq::from(8)]);

        layer.get_add_i_and_mul_i::<Fq>(1);
    }
}
