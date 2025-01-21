// use std::io;

// //for sparsee
// struct UnivariatePoly{
//     coeffecient:Vec<(u32, u32)>
// }

// impl UnivariatePoly{
//     fn new( coeffecient: Vec<(u32, u32)>) -> UnivariatePoly{
//         Self{
//             coeffecient:  coeffecient
//         }
//     }

//     fn degree(&self) -> u32{
//         let degree = self.coeffecient.iter().map(|(_, x)| x).max().unwrap();
//         *degree
//     }

//     fn evaluate(&self, x:u32) -> u32{
//         let result:u32 = self.coeffecient.iter().map(|(c,d) | c * x.pow(*d)).sum();
//         result
//     }
// }

use polynomial::implementation::shamir_secret_sharing;
use ark_bn254::Fq;
fn main() {
    shamir_secret_sharing::create_secret(Fq::from(4),4,10);
}
