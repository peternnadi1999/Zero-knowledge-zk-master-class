use ark_ff::PrimeField;
use crate::multilinear_poly::mult_polynomial::MultilinearPoly;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductPoly<F: PrimeField> {
    pub poly: Vec<MultilinearPoly<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    pub fn new(eval_poly: Vec<MultilinearPoly<F>>) -> Self {
        let first_len = eval_poly.first().map(|x| x.evaluated_value.len());

        if eval_poly.iter().any(|x| Some(x.evaluated_value.len()) != first_len) {
            panic!("Length of evaluated_value is not equal");
        }

        let poly = eval_poly
            .iter()
            .map(|x| x.clone())
            .collect();

        Self {
            poly,
        }
    }

    pub fn partial_evaluate(&self, index: usize, value: F) -> Self {
        let partial_eval = self.poly
            .iter()
            .map(|poly| poly.partial_evaluate(index, value))
            .collect();

        Self {
            poly: partial_eval,
        }
    }

    pub fn evaluate(&self, values: Vec<F>) -> F {
        self.poly
            .iter()
            .map(|poly| poly.clone().evaluate_poly(&values).expect("error"))
            .product()
    }

    pub fn reduce(&self) -> Vec<F> { 
        let result: Vec<F> = self.poly[0].evaluated_value
            .iter()
            .zip(self.poly[1].evaluated_value.iter())
            .map(|(x, y)| *x * *y)
            .collect();
        result

    }

    pub fn get_degree(&self) -> usize {
        self.poly.len()
    }
}



#[cfg(test)]
mod tests {
    use ark_bn254::Fq;
    use crate::multilinear_poly::mult_polynomial::MultilinearPoly;
    use super::ProductPoly;

    #[test]
    fn test_product_poly() {
        let a = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let poly_1 = MultilinearPoly::new(a.clone(), a.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let result = vec![poly_1, poly_2];
        let product_poly = ProductPoly::new(result);
        // assert!(product_poly);
    }

    #[test]
    fn test_partial_evaluation(){
        let a = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let poly_1 = MultilinearPoly::new(a.clone(), a.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let result = vec![poly_1, poly_2];
        let product_poly = ProductPoly::new(result);
        let partial_evaluate = product_poly.partial_evaluate(0, Fq::from(2));
        dbg!(partial_evaluate);
    }

    #[test]
    fn test_evaluation(){
        let a = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let poly_1 = MultilinearPoly::new(a.clone(), a.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let result = vec![poly_1, poly_2];
        let product_poly = ProductPoly::new(result);
        let evaluate = product_poly.evaluate(vec![Fq::from(1), Fq::from(2)]);
        dbg!(evaluate);
    }

    #[test]
    fn test_reduce(){
        let a = vec![Fq::from(0), Fq::from(3), Fq::from(2), Fq::from(5)];
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let poly_1 = MultilinearPoly::new(a.clone(), a.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();
        let result = vec![poly_1, poly_2];
        let product_poly = ProductPoly::new(result);
       
        let partial_evaluate = product_poly.partial_evaluate(0, Fq::from(2));

        let reduce = partial_evaluate.reduce();
        assert_eq!(reduce, vec![Fq::from(20), Fq::from(42)]);
    }
}
