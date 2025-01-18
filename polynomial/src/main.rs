use std::io;

//for sparsee
struct UnivariatePoly{
    coeffecient:Vec<(u32, u32)>
}

impl UnivariatePoly{
    fn new( coeffecient: Vec<(u32, u32)>) -> UnivariatePoly{
        Self{
            coeffecient:  coeffecient
        }
    }

    fn degree(&self) -> u32{
        let degree = self.coeffecient.iter().map(|(_, x)| x).max().unwrap();
        *degree
    }

    fn evaluate(&self, x:u32) -> u32{
        let result:u32 = self.coeffecient.iter().map(|(c,d) | c * x.pow(*d)).sum();
        result 
    }
}


// for Dense
// struct UnivariatePoly{
//         coeffecient:Vec<f64>
//     }
    
//     impl UnivariatePoly{
//         fn degree(&self) -> usize{
//             self.coeffecient.len() - 1
//         }

//         fn evaluate(&self, x:f64) ->f64{
//             self.coeffecient.len().iter().enumerate().map(|(i, coeff)| { coeff * x.powf(i as f64)}).sum()
//         }
//     }


fn main() {
    // lagrange(vec![(0.0,0.0), (1.0,5.0), (2.0,14.0)], 3.0);
    println!("Please input the number of monomial.");
    let mut monomial = String::new();
    
    io::stdin()
    .read_line(&mut monomial)
    .expect("Failed to read line");
    
  
    let monomial = monomial.trim().parse().expect("invalid number");
    let mut terms = Vec::new();

    for count in 0..monomial {
        
            println!("Please input coefficient {} value.", count+1);
            let mut coeffecient = String::new();
            
                io::stdin()
                    .read_line(&mut coeffecient)
                    .expect("Failed to read line");
                    let coeffecient = coeffecient.trim().parse().expect("Invalid coefficient");
   
        
            println!("Please input exponent {} value.", count+1);
            let mut exponent = String::new();
                io::stdin()
                    .read_line(&mut exponent)
                    .expect("Failed to read line");
                    let exponent = exponent.trim().parse().expect("Invalid exponent");

            terms.push((coeffecient, exponent));
        }

    
    
    let polynomial = UnivariatePoly::new(terms);

    let mut x = String::new(); 

    println!("Please input the x value."); 
        io::stdin()
        .read_line(&mut x)
        .expect("Failed to read line");

    let x = x.trim().parse().expect("Invalid x value");

    println!("The degree is = {:?}", polynomial.degree());
    println!("F(x) is = {:?} ", polynomial.evaluate(x));
}



// fn lagrange(points: Vec<(f64, f64)>, x:f64) {

//     let mut result = 0.0;
    
//     for i in 0..points.len(){
//         let mut term = points[i].1;
//         println!("i = {}", points[i].0);
//         println!("x = {}", points[i].1);
//             for j in 0..points.len(){
//                 if i != j {
                   
//                     term = term * ((x - points[j].0) / ( points[i].0 - points[j].0));
//                     println!("j = {},s={} term {}", points[j].0,(x - points[j].0), term);
//                 }
//             }
//             result += term;
//             println!("result ={}", result);
//     }
 

// }

