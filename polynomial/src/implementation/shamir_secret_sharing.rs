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


fn recover_secret<F: PrimeField>(shares: Vec<F>, threshold: u64 ) {
    todo!()
}


#[cfg(test)]

mod tests {
    use ark_bn254::Fq;
    use super::*;

    #[test]
    fn test_create_secret(){
       let secret = 10;
       let threshold = 3;
       let total_share = 5;
       let shares = create_secret(Fq::from(secret), threshold, total_share);
       println!("{:?}", shares);
       assert_eq!(shares.len(), total_share as usize);
    }

    fn test_ecover_secret(){
        todo!()

    }
}
