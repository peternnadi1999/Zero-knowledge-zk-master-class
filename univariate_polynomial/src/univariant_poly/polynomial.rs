use std::iter::{Product, Sum};
use std::ops::{Add, Mul};
use ark_ff::PrimeField;

#[derive(Debug, PartialEq, Clone)]
pub struct  UnivariatePoly<F: PrimeField> {
    // 1 coefficient for each power of x
    coefficient: Vec<F>,
}

impl <F: PrimeField> UnivariatePoly<F> {
   pub fn new(coefficient: Vec<F>) -> Self {
        UnivariatePoly { coefficient }
    }

    fn degree(&self) -> usize {
        self.coefficient.len() - 1
    }


    pub fn evaluate(&self, x: F) -> F {
        // let mut evaluation = 0.0;
        // let mut current_x = 1.0;
        // for i in 0..self.coefficient.len() {
        //     evaluation += self.coefficient[i] * current_x;
        //     current_x *= x;
        // }
        // evaluation

        // c1 + c2*x + c3*x*x -> 3 mul
        // c1 + x*(c2 + c3*x) -> 2 mul

        self.coefficient
            .iter()
            .rev()
            .cloned()
            .reduce(|acc, curr| acc * x + curr)
            .unwrap()

        // self.coefficient
        //     .iter()
        //     .enumerate()
        //     .map(|(i, coeff)| coeff * x.powf(i as f64))
        //     .sum()
    }

    pub fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
        xs.iter()
            .zip(ys.iter())
            .map(|(x, y)| Self::basis(x, &xs).scalar_mul(y))
            .sum()
    }

    fn scalar_mul(&self, scalar: &F) -> Self {
        UnivariatePoly {
            coefficient: self
                .coefficient
                .iter()
                .map(|coeff| *coeff * *scalar)
                .collect(),
        }
    }

    fn basis(x: &F, interpolating_set: &[F]) -> Self {
        // numerator
        let numerator: UnivariatePoly<F> = interpolating_set
            .iter()
            .filter(|val| *val != x)
            .map(|x_n| UnivariatePoly::new(vec![F::neg(*x_n), F::one()]))
            .product();

        // denominator
        let denominator = F::one() /  numerator.evaluate(*x);

        numerator.scalar_mul(&denominator)
    }
}

impl <F: PrimeField>Mul for &UnivariatePoly<F> {
    type Output = UnivariatePoly<F>;

    fn mul(self, rhs: Self) -> Self::Output {
        // mul for dense
        let new_degree = self.degree() + rhs.degree();
        let mut result = vec![F::zero(); new_degree + 1];
        for i in 0..self.coefficient.len() {
            for j in 0..rhs.coefficient.len() {
                result[i + j] += self.coefficient[i] * rhs.coefficient[j]
            }
        }
        UnivariatePoly {
            coefficient: result,
        }
    }
}

impl <F: PrimeField>Add for &UnivariatePoly<F> {
    type Output = UnivariatePoly<F>;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.degree() < rhs.degree() {
            (rhs.clone(), self)
        } else {
            (self.clone(), rhs)
        };

        let _ = bigger
            .coefficient
            .iter_mut()
            .zip(smaller.coefficient.iter())
            .map(|(b_coeff, s_coeff)| *b_coeff += s_coeff)
            .collect::<()>();

        UnivariatePoly::new(bigger.coefficient)
    }
}

impl <F: PrimeField>Sum for UnivariatePoly<F>{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = UnivariatePoly::new(vec![F::zero()]);
        for poly in iter {
            result = &result + &poly;
        }
        result
    }
}

impl <F: PrimeField>Product for UnivariatePoly<F> {
    fn product<I: Iterator<Item=Self>>(iter: I) -> Self {
        let mut result = UnivariatePoly::new(vec![F::one()]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fq;

    fn poly_1() -> UnivariatePoly<Fq>{
        // f(x) = 1 + 2x + 3x^2
        UnivariatePoly {
            coefficient: vec![ Fq::from(1), Fq::from(2), Fq::from(3)],
        }
    }

    fn poly_2() -> UnivariatePoly<Fq> {
        // f(x) = 4x + 3 + 5x^11
        UnivariatePoly {
            coefficient: [vec![Fq::from(3), Fq::from(4)], vec![Fq::from(0), Fq::from(9)], vec![Fq::from(5)]].concat(),
        }
    }

    #[test]
    fn test_degree() {
        assert_eq!(poly_1().degree(), 2);
    }

    #[test]
    fn test_evaluation() {
        assert_eq!(poly_1().evaluate(Fq::from(2)), Fq::from(17));
    }

    #[test]
    fn test_addition() {
        // f(x) = 1 + 2x + 3x^2
        // f(x) = 4x + 3 + 5x^11

        // r(x) = 4 + 6x + 3x^2 + 5x^11
        assert_eq!(
            (&poly_1() + &poly_2()).coefficient,
            [vec![Fq::from(4), Fq::from(6), Fq::from(3)], vec![Fq::from(0), Fq::from(8)], vec![Fq::from(5)]].concat()
        )
    }

    #[test]
    fn test_mul() {
        // f(x) = 5 + 2x^2
        let poly_1 = UnivariatePoly {
            coefficient: vec![Fq::from(5), Fq::from(0), Fq::from(2)],
        };
        // f(x) = 2x + 6
        let poly_2 = UnivariatePoly {
            coefficient: vec![Fq::from(6), Fq::from(2)],
        };

        // r(x) = 30 + 10x + 12x^2 + 4x^3
        assert_eq!((&poly_1 * &poly_2).coefficient, vec![Fq::from(30), Fq::from(10), Fq::from(12), Fq::from(4)]);
    }

    #[test]
    fn test_interpolate() {
        // f(x) = 2x
        // [(2, 4), (4, 8)]
        let maybe_2x = UnivariatePoly::interpolate(vec![Fq::from(2), Fq::from(4)], vec![Fq::from(4), Fq::from(8)]);
        assert_eq!(
            maybe_2x.coefficient,
            vec![Fq::from(0), Fq::from(2)]
        );
    }
}