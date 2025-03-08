use ark_ff::PrimeField;
use super::product_poly::ProductPoly;

#[derive(Debug, Clone)]
pub struct SumPoly<F: PrimeField> {
    pub poly: Vec<ProductPoly<F>>,
}

impl<F: PrimeField> SumPoly<F> {
    pub fn new(poly: Vec<ProductPoly<F>>) -> Self {
        let first_len = poly.first().map(|x| x.get_degree());

        if poly.iter().any(|x| Some(x.get_degree()) != first_len) {
            panic!("all product polys must have same degree");
        }

        Self {
            poly,
        }
    }

    pub fn evaluate(&self, x: Vec<F>) -> F {
        self.poly
            .iter()
            .map(|poly| poly.evaluate(x.clone()))
            .sum()
    }

    pub fn partial_evaluate(&self, index: usize, value: F) -> Self {
        let partial_polys = self.poly
            .iter()
            .map(|product_poly| product_poly.partial_evaluate(index, value))
            .collect();

        Self { poly: partial_polys }
    }

    pub fn reduce(&self) -> Vec<F> {
        let x = self.poly[0].clone().reduce();
        let y = self.poly[1].clone().reduce();

        let result: Vec<F> = x
            .iter()
            .zip(y.iter())
            .map(|(x, y)| *x + *y)
            .collect();
        result
    }

    pub fn degree(&self) -> usize {
        self.poly[0].get_degree()
    }
}

#[cfg(test)]
mod tests {

    use crate::multilinear_poly::mult_polynomial::MultilinearPoly;

    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_sumpoly() {
        let a = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let b = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];

        let c = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
        let d = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];

        let poly_1 = MultilinearPoly::new(a.clone(), a.len().ilog2() as usize).unwrap();
        let poly_2 = MultilinearPoly::new(b.clone(), b.len().ilog2() as usize).unwrap();

        let poly_3 = MultilinearPoly::new(c.clone(), b.len().ilog2() as usize).unwrap();
        let poly_4 = MultilinearPoly::new(d.clone(), c.len().ilog2() as usize).unwrap();

        let product_poly1 = ProductPoly::new(vec![poly_1.clone(), poly_2.clone()]);
        let product_poly2 = ProductPoly::new(vec![poly_3.clone(), poly_4.clone()]);

        let sum_poly = SumPoly::new(vec![product_poly1, product_poly2]);
        // assert!(sum_poly);
        
    }
}
