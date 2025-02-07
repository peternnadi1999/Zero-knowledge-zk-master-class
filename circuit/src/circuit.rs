use ark_ff::PrimeField;

#[derive(Debug)]
struct Gate<F: PrimeField> {
    output: F,
    left_index: usize,    // Index for the left input
    right_index: usize,   // Index for the right input
    opt: Opt,
}

#[derive(Debug)]
enum Opt {
    Add,
    Mul,
}

#[derive(Debug)]
struct Layer<F: PrimeField> {
    gates: Vec<Gate<F>>,
}

#[derive(Debug)]
struct Circuit<F: PrimeField> {
    layers: Vec<Layer<F>>,
}

impl<F: PrimeField> Gate<F> {
    fn new(left_index: usize, right_index: usize, opt: Opt) -> Self {
        Self {
            output: F::from(0u32), // Placeholder, will be updated after evaluation
            left_index,
            right_index,
            opt,
        }
    }

    fn evaluate(&mut self, inputs: &[F]) {
        let left_input = inputs[self.left_index];
        let right_input = inputs[self.right_index];

        self.output = match self.opt {
            Opt::Add => left_input + right_input,
            Opt::Mul => left_input * right_input,
        };
    }
}

impl<F: PrimeField> Layer<F> {
    fn new(gates: Vec<Gate<F>>) -> Self {
        Self { gates }
    }

    fn evaluate(&mut self, inputs: &[F]) -> Vec<F> {
        for gate in &mut self.gates {
            gate.evaluate(inputs);
        }
        self.gates.iter().map(|gate| gate.output).collect()
    }
}

impl<F: PrimeField> Circuit<F> {
    fn new(layers: Vec<Layer<F>>) -> Self {
        Self { layers }
    }

    fn evaluate(&mut self, initial_inputs: Vec<F>) -> Vec<F> {
        let mut current_inputs = initial_inputs;
        for layer in &mut self.layers {
            current_inputs = layer.evaluate(&current_inputs);
        }
        current_inputs
    }
}

#[cfg(test)]
mod tests {
    use ark_bn254::Fq;
    use super::*;

    #[test]
    fn test_gate_evaluation() {
        let mut add_gate = Gate::new(0, 1, Opt::Add);
        add_gate.evaluate(&[Fq::from(2), Fq::from(3)]);
        assert_eq!(add_gate.output, Fq::from(5));

        let mut mul_gate = Gate::new(0, 1, Opt::Mul);
        mul_gate.evaluate(&[Fq::from(2), Fq::from(8)]);
        assert_eq!(mul_gate.output, Fq::from(16));
    }

    #[test]
    fn test_layer_evaluation() {
        let gates = vec![
            Gate::new(0, 1, Opt::Add),
            Gate::new(0, 1, Opt::Mul),
            Gate::new(2, 1, Opt::Add), // Uses output of the first gate and an external input
        ];

        let mut layer = Layer::new(gates);
        let outputs = layer.evaluate(&[Fq::from(2), Fq::from(3), Fq::from(4)]);
        assert_eq!(outputs, vec![Fq::from(5), Fq::from(6), Fq::from(7)]);
    }

    #[test]
    fn test_circuit_evaluation() {
        let layer1 = Layer::new(vec![
            Gate::new(0, 1, Opt::Add), // 2 + 3 = 5
            Gate::new(0, 1, Opt::Mul), // 2 * 3 = 6
        ]);

        let layer2 = Layer::new(vec![
            Gate::new(0, 1, Opt::Add), // 5 + 6 = 11
        ]);

        let mut circuit = Circuit::new(vec![layer1, layer2]);
        let outputs = circuit.evaluate(vec![Fq::from(2), Fq::from(3)]);

        assert_eq!(outputs, vec![Fq::from(11)]);
    }
}
