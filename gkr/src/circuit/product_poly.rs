
use ark_ff::PrimeField;
use multilinear::multilinear_poly::mult_polynomial::MultilinearPoly;


pub struct ProducPoly<F: PrimeField> {
    pub poly: Vec<MultilinearPoly<F>>,
}

pub struct SumPoly<F: PrimeField>{
    pub poly: Vec<MultilinearPoly<F>>,
}

impl<F: PrimeField> ProducPoly<F> {
   pub  fn new(self, poly: Vec<MultilinearPoly<F>>) -> Self {
        Self {
            poly,
        }
    }

    fn product_poly(&self) -> MultilinearPoly<F> {
        todo!()
    }
}

impl<F: PrimeField> SumPoly<F> {
   pub fn new(self, poly: Vec<MultilinearPoly<F>>) -> Self {
        Self {
            poly
        }
    }

    fn sum_poly(&self) -> MultilinearPoly<F> {
        todo!()
    }
}



pub fn tensor_addition<F: PrimeField>(
    b: &MultilinearPoly<F>,
    c: &MultilinearPoly<F>
) -> MultilinearPoly<F> {
    let result: Vec<F> = b.evaluated_value
        .iter()
        .flat_map(|&x| c.evaluated_value.iter().map(move |&y| x + y))
        .collect();
    dbg!(&result);
    MultilinearPoly::new(result.clone(), result.len().ilog2() as usize).expect(
        "Failed to create MultilinearPoly"
    )
}

pub fn tensor_multiplication<F: PrimeField>(
    b: &MultilinearPoly<F>,
    c: &MultilinearPoly<F>
) -> MultilinearPoly<F> {
    let result: Vec<F> = b.evaluated_value
        .iter()
        .flat_map(|&x| c.evaluated_value.iter().map(move |&y| x * y))
        .collect();
    MultilinearPoly::new(result.clone(), result.len().ilog2() as usize).expect(
        "Failed to create MultilinearPoly"
    )
}

