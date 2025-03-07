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


fn main() {
    print!("hello world");
}
