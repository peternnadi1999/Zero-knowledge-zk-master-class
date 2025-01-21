use rand::prelude::*;
use crate::implementation::polynomial::UnivariatePoly;
use ark_ff::PrimeField;

pub fn create_secret<F: PrimeField>(secret: F, threshold: u64, total_share: u64) -> Vec<F> {
    let  mut ys = vec![secret];
    let mut rng = rand::thread_rng();
    for _i in 0..threshold - 1{
        let y  = F::from(rng.gen_range(1..4));
        ys.push(y);
    }

    let poly = UnivariatePoly::new(ys);

    let mut  shares = Vec::new();

    for _j in 0..total_share {
        let x  = F::from(rng.gen_range(1..4));
        let evaluate = poly.evaluate(x);
        shares.push(evaluate);

    }
shares
    
}


fn recover_secret<F: PrimeField>() {
    todo!()
}


#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_create_secret(){
       todo!()
    }
}
