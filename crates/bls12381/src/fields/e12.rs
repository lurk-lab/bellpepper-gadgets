use bellpepper_core::boolean::{AllocatedBit, Boolean};
use bellpepper_core::{ConstraintSystem, SynthesisError};
use bls12_381::fp12::Fp12 as BlsFp12;
use bls12_381::fp6::Fp6 as BlsFp6;
use ff::{PrimeField, PrimeFieldBits};

use super::e2::AllocatedE2Element;
use super::e6::AllocatedE6Element;

#[derive(Clone)]
pub struct AllocatedE12Element<F: PrimeField + PrimeFieldBits> {
    pub(crate) c0: AllocatedE6Element<F>,
    pub(crate) c1: AllocatedE6Element<F>,
}

impl<F> From<&BlsFp12> for AllocatedE12Element<F>
where
    F: PrimeField + PrimeFieldBits,
{
    fn from(value: &BlsFp12) -> Self {
        let c0 = AllocatedE6Element::<F>::from(&value.c0);
        let c1 = AllocatedE6Element::<F>::from(&value.c1);
        Self { c0, c1 }
    }
}

impl<F> From<&AllocatedE12Element<F>> for BlsFp12
where
    F: PrimeField + PrimeFieldBits,
{
    fn from(value: &AllocatedE12Element<F>) -> Self {
        let c0 = BlsFp6::from(&value.c0);
        let c1 = BlsFp6::from(&value.c1);
        BlsFp12 { c0, c1 }
    }
}

impl<F: PrimeField + PrimeFieldBits> AllocatedE12Element<F> {
    pub fn zero() -> Self {
        Self {
            c0: AllocatedE6Element::zero(),
            c1: AllocatedE6Element::zero(),
        }
    }

    pub fn one() -> Self {
        Self {
            c0: AllocatedE6Element::one(),
            c1: AllocatedE6Element::zero(),
        }
    }

    pub fn alloc_element<CS>(cs: &mut CS, value: &BlsFp12) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let c0 =
            AllocatedE6Element::<F>::alloc_element(&mut cs.namespace(|| "allocate c0"), &value.c0)?;
        let c1 =
            AllocatedE6Element::<F>::alloc_element(&mut cs.namespace(|| "allocate c1"), &value.c1)?;

        Ok(Self { c0, c1 })
    }

    pub fn assert_is_equal<CS>(cs: &mut CS, a: &Self, b: &Self) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        AllocatedE6Element::<F>::assert_is_equal(&mut cs.namespace(|| "c0 =? c0"), &a.c0, &b.c0)?;
        AllocatedE6Element::<F>::assert_is_equal(&mut cs.namespace(|| "c1 =? c1"), &a.c1, &b.c1)?;
        Ok(())
    }

    pub fn reduce<CS>(&self, cs: &mut CS) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let c0_reduced = self.c0.reduce(&mut cs.namespace(|| "c0 mod P"))?;
        let c1_reduced = self.c1.reduce(&mut cs.namespace(|| "c1 mod P"))?;
        Ok(Self {
            c0: c0_reduced,
            c1: c1_reduced,
        })
    }

    pub fn assert_equality_to_constant<CS>(
        &self,
        cs: &mut CS,
        constant: &Self,
    ) -> Result<(), SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        self.c0
            .assert_equality_to_constant(&mut cs.namespace(|| "c0 =? (const) c0"), &constant.c0)?;
        self.c1
            .assert_equality_to_constant(&mut cs.namespace(|| "c1 =? (const) c1"), &constant.c1)?;
        Ok(())
    }

    pub fn alloc_is_zero<CS>(&self, cs: &mut CS) -> Result<AllocatedBit, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let z0 = self.c0.alloc_is_zero(&mut cs.namespace(|| "c0 =? 0"))?;
        let z1 = self.c1.alloc_is_zero(&mut cs.namespace(|| "c1 =? 0"))?;

        AllocatedBit::and(&mut cs.namespace(|| "and(z0, z1)"), &z0, &z1)
    }

    pub fn add<CS>(&self, cs: &mut CS, value: &Self) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let c0 = self
            .c0
            .add(&mut cs.namespace(|| "compute c0 + c0"), &value.c0)?;
        let c1 = self
            .c1
            .add(&mut cs.namespace(|| "compute c1 + c1"), &value.c1)?;
        Ok(Self { c0, c1 })
    }

    pub fn sub<CS>(&self, cs: &mut CS, value: &Self) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let c0 = self
            .c0
            .sub(&mut cs.namespace(|| "compute c0 - c0"), &value.c0)?;
        let c1 = self
            .c1
            .sub(&mut cs.namespace(|| "compute c1 - c1"), &value.c1)?;
        Ok(Self { c0, c1 })
    }

    pub fn conjugate<CS>(&self, cs: &mut CS) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let z1 = self.c1.neg(&mut cs.namespace(|| "conj e12"))?;
        Ok(Self {
            c0: self.c0.clone(),
            c1: z1,
        })
    }

    pub fn mul<CS>(&self, cs: &mut CS, value: &Self) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let (x, y) = (self, value);
        let mut cs = cs.namespace(|| "compute e12 mul(x,y)");
        let a = x.c0.add(&mut cs.namespace(|| "a <- x.c0 + x.c1"), &x.c1)?;
        let b = y.c0.add(&mut cs.namespace(|| "b <- y.c0 + y.c1"), &y.c1)?;
        let a = a.mul(&mut cs.namespace(|| "a <- a * b"), &b)?;
        let b = x.c0.mul(&mut cs.namespace(|| "b <- x.c0 * y.c0"), &y.c0)?;
        let c = x.c1.mul(&mut cs.namespace(|| "c <- x.c1 * y.c1"), &y.c1)?;
        let z1 = a.sub(&mut cs.namespace(|| "z1 <- a - b"), &b)?;
        let z1 = z1.sub(&mut cs.namespace(|| "z1 <- z1 - c"), &c)?;
        let z0 = c.mul_by_nonresidue(&mut cs.namespace(|| "z0 <- c.mul_by_nonresidue()"))?;
        let z0 = z0.add(&mut cs.namespace(|| "z0 <- z0 + b"), &b)?;

        Ok(Self { c0: z0, c1: z1 })
    }

    /// MulBy014 multiplies z by an E12 sparse element of the form
    ///
    ///  E12{
    ///    C0: E6{B0: c0, B1: c1, B2: 0},
    ///    C1: E6{B0: 0, B1: 1, B2: 0},
    ///  }
    pub fn mul_by_014<CS>(
        &self,
        cs: &mut CS,
        c0: &AllocatedE2Element<F>,
        c1: &AllocatedE2Element<F>,
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let z = self;
        let mut cs = cs.namespace(|| "compute e12 mul_by_014(c0, c1)");

        let a =
            z.c0.mul_by_01(&mut cs.namespace(|| "a <- z.c0.mul_by_01(c0, c1)"), c0, c1)?;

        let b = AllocatedE6Element {
            b0: z
                .c1
                .b2
                .mul_by_nonresidue(&mut cs.namespace(|| "b.b0 <- z.c1.b2.mul_by_nonresidue()"))?,
            b1: z.c1.b0.clone(),
            b2: z.c1.b1.clone(),
        };

        let one = AllocatedE2Element::<F>::one();
        let d = c1.add(&mut cs.namespace(|| "d <- c1 + 1"), &one)?;

        let rc1 =
            z.c1.add(&mut cs.namespace(|| "rc1 <- z.c1 + z.c0"), &z.c0)?;
        let rc1 = rc1.mul_by_01(&mut cs.namespace(|| "rc1 <- rc1.mul_by_01(c0, d)"), c0, &d)?;
        let rc1 = rc1.sub(&mut cs.namespace(|| "rc1 <- rc1 - a"), &a)?;
        let rc1 = rc1.sub(&mut cs.namespace(|| "rc1 <- rc1 - b"), &b)?;
        let rc0 = b.mul_by_nonresidue(&mut cs.namespace(|| "rc0 <- b.mul_by_nonresidue()"))?;
        let rc0 = rc0.add(&mut cs.namespace(|| "rc0 <- rc0 + a"), &a)?;

        Ok(Self { c0: rc0, c1: rc1 })
    }

    ///  mul_014_by_014 multiplies two E12 sparse element of the form:
    ///
    ///  E12{
    ///    C0: E6{B0: c0, B1: c1, B2: 0},
    ///    C1: E6{B0: 0, B1: 1, B2: 0},
    ///  }
    ///
    /// and
    ///
    ///  E12{
    ///    C0: E6{B0: d0, B1: d1, B2: 0},
    ///    C1: E6{B0: 0, B1: 1, B2: 0},
    ///  }
    pub fn mul_014_by_014<CS>(
        cs: &mut CS,
        c0: &AllocatedE2Element<F>,
        c1: &AllocatedE2Element<F>,
        d0: &AllocatedE2Element<F>,
        d1: &AllocatedE2Element<F>,
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let mut cs = cs.namespace(|| "compute e12 mul_014_by_014(c0, c1, d0, d1)");
        let one = AllocatedE2Element::<F>::one();
        let x0 = c0.mul(&mut cs.namespace(|| "x0 <- c0 * d0"), d0)?;
        let x1 = c1.mul(&mut cs.namespace(|| "x1 <- c1 * d1"), d1)?;
        let tmp = c0.add(&mut cs.namespace(|| "tmp <- c0 + 1"), &one)?;
        let x04 = d0.add(&mut cs.namespace(|| "x04 <- d0 + 1"), &one)?;
        let x04 = x04.mul(&mut cs.namespace(|| "x04 <- x04 * tmp"), &tmp)?;
        let x04 = x04.sub(&mut cs.namespace(|| "x04 <- x04 - x0"), &x0)?;
        let x04 = x04.sub(&mut cs.namespace(|| "x04 <- x04 - 1"), &one)?;
        let tmp = c0.add(&mut cs.namespace(|| "tmp <- c0 + c1"), c1)?;
        let x01 = d0.add(&mut cs.namespace(|| "x01 <- d0 + d1"), d1)?;
        let x01 = x01.mul(&mut cs.namespace(|| "x01 <- x01 * tmp"), &tmp)?;
        let x01 = x01.sub(&mut cs.namespace(|| "x01 <- x01 - x0"), &x0)?;
        let x01 = x01.sub(&mut cs.namespace(|| "x01 <- x01 - x1"), &x1)?;
        let tmp = c1.add(&mut cs.namespace(|| "tmp <- c1 + 1"), &one)?;
        let x14 = d1.add(&mut cs.namespace(|| "x14 < - d1 + 1"), &one)?;
        let x14 = x14.mul(&mut cs.namespace(|| "x14 <- x14 * tmp"), &tmp)?;
        let x14 = x14.sub(&mut cs.namespace(|| "x14 <- x14 - x1"), &x1)?;
        let x14 = x14.sub(&mut cs.namespace(|| "x14 <- x14 - 1"), &one)?;

        let zc0b0 = AllocatedE2Element::<F>::non_residue();
        let zc0b0 = zc0b0.add(&mut cs.namespace(|| "zc0b0 <- non_residue() + x0"), &x0)?;

        // TODO: is this usage without calling alloc_element correct?
        Ok(AllocatedE12Element {
            c0: AllocatedE6Element {
                b0: zc0b0,
                b1: x01,
                b2: x1,
            },
            c1: AllocatedE6Element {
                b0: AllocatedE2Element::<F>::zero(),
                b1: x04,
                b2: x14,
            },
        })
    }

    /// mul_by_01245 multiplies z by an E12 sparse element of the form
    ///
    ///  E12{
    ///    C0: E6{B0: c00, B1: c01, B2: c02},
    ///    C1: E6{B0: 0, B1: c11, B2: c12},
    ///  }
    pub fn mul_by_01245<CS>(
        &self,
        cs: &mut CS,
        c00: &AllocatedE2Element<F>,
        c01: &AllocatedE2Element<F>,
        c02: &AllocatedE2Element<F>,
        c11: &AllocatedE2Element<F>,
        c12: &AllocatedE2Element<F>,
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let z = self;
        let c0 = AllocatedE6Element {
            b0: c00.clone(),
            b1: c01.clone(),
            b2: c02.clone(),
        };
        let c1 = AllocatedE6Element {
            b0: AllocatedE2Element::zero(),
            b1: c11.clone(),
            b2: c12.clone(),
        };
        let mut cs = cs.namespace(|| "compute e12 mul_by_01245(x, e12(c0,c1))");
        let a = z.c0.add(&mut cs.namespace(|| "a <- z.c0 + z.c1"), &z.c1)?;
        let b = c0.add(&mut cs.namespace(|| "b <- c0 + c1"), &c1)?;
        let a = a.mul(&mut cs.namespace(|| "a <- a * b"), &b)?;
        let b = z.c0.mul(&mut cs.namespace(|| "b <- z.c0 * c0"), &c0)?;
        let c = z.c1.mul_by_12(
            &mut cs.namespace(|| "c <- z.c1.mul_by_12(c11, c12)"),
            c11,
            c12,
        )?;
        let z1 = a.sub(&mut cs.namespace(|| "z1 <- a - b"), &b)?;
        let z1 = z1.sub(&mut cs.namespace(|| "z1 <- z1 - c"), &c)?;
        let z0 = c.mul_by_nonresidue(&mut cs.namespace(|| "z0 <- c.mul_by_nonresidue()"))?;
        let z0 = z0.add(&mut cs.namespace(|| "z0 <- z0 + b"), &b)?;

        Ok(Self { c0: z0, c1: z1 })
    }

    pub fn square<CS>(&self, cs: &mut CS) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let x = self;
        let mut cs = cs.namespace(|| "compute e12 square(x)");
        let c0 = x.c0.sub(&mut cs.namespace(|| "c0 <- x.c0 - x.c1"), &x.c1)?;
        let c3 =
            x.c1.mul_by_nonresidue(&mut cs.namespace(|| "c3 <- x.c1.mul_by_nonresidue()"))?;
        let c3 = c3.neg(&mut cs.namespace(|| "c3 <- c3.neg()"))?;
        let c3 = x.c0.add(&mut cs.namespace(|| "c3 <- x.c0 + c3"), &c3)?;
        let c2 = x.c0.mul(&mut cs.namespace(|| "c2 <- x.c0 * x.c1"), &x.c1)?;
        let c0 = c0.mul(&mut cs.namespace(|| "c0 <- c0 * c3"), &c3)?;
        let c0 = c0.add(&mut cs.namespace(|| "c0 <- c0 + c2"), &c2)?;
        let z1 = c2.double(&mut cs.namespace(|| "z1 <- c2.double()"))?;
        let c2 = c2.mul_by_nonresidue(&mut cs.namespace(|| "c2 <- c2.mul_by_nonresidue()"))?;
        let z0 = c0.add(&mut cs.namespace(|| "z0 <- c0 + c2"), &c2)?;

        Ok(Self { c0: z0, c1: z1 })
    }

    pub fn inverse<CS>(&self, cs: &mut CS) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let val = BlsFp12::from(self);
        if val.is_zero().into() {
            eprintln!("Inverse of zero element cannot be calculated");
            return Err(SynthesisError::DivisionByZero);
        }
        let inv = val.invert().unwrap();

        let inv_alloc = Self::alloc_element(&mut cs.namespace(|| "alloc inv"), &inv)?;

        // x*inv = 1
        let prod = inv_alloc.mul(&mut cs.namespace(|| "x*inv"), self)?;

        // CLEANUP: do we need to reduce here (and add the width constraints and etc) or would compute_rem be enough?
        // can't really assert equality to constant here without reducing it mod P, but this has more constraints than
        // just using assert_is_equal instead of assert_equality_to_constant

        // let prod = prod.reduce(&mut cs.namespace(|| "x*inv mod P"))?;
        // prod.assert_equality_to_constant(&mut cs.namespace(|| "x*inv = 1"), &Self::one())?;

        Self::assert_is_equal(&mut cs.namespace(|| "x*inv = 1 mod P"), &prod, &Self::one())?;

        Ok(inv_alloc)
    }

    pub fn div_unchecked<CS>(&self, cs: &mut CS, value: &Self) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        // returns self/value (or x/y)

        let x = BlsFp12::from(self);

        let y = BlsFp12::from(value);
        if y.is_zero().into() {
            eprintln!("Inverse of zero element cannot be calculated");
            return Err(SynthesisError::DivisionByZero);
        }
        let y_inv = y.invert().unwrap();
        let div = y_inv * x;

        let div_alloc = Self::alloc_element(&mut cs.namespace(|| "alloc div"), &div)?;

        // y*div = x
        let prod = div_alloc.mul(&mut cs.namespace(|| "y*div"), value)?;
        Self::assert_is_equal(&mut cs.namespace(|| "y*div = x"), &prod, self)?;

        Ok(div_alloc)
    }

    pub fn conditionally_select<CS>(
        cs: &mut CS,
        z0: &Self,
        z1: &Self,
        condition: &Boolean,
    ) -> Result<Self, SynthesisError>
    where
        CS: ConstraintSystem<F>,
    {
        let c0 = AllocatedE6Element::<F>::conditionally_select(
            &mut cs.namespace(|| "cond b0"),
            &z0.c0,
            &z1.c0,
            condition,
        )?;
        let c1 = AllocatedE6Element::<F>::conditionally_select(
            &mut cs.namespace(|| "cond b1"),
            &z0.c1,
            &z1.c1,
            condition,
        )?;
        Ok(Self { c0, c1 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bellpepper_core::test_cs::TestConstraintSystem;
    use pasta_curves::Fp;

    #[test]
    fn test_random_add() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let b = BlsFp12::random(&mut rng);
        let c = &a + &b;

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let b_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc b"), &b);
        assert!(b_alloc.is_ok());
        let b_alloc = b_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.add(&mut cs.namespace(|| "a+b"), &b_alloc);
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a+b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 3144);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_sub() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let b = BlsFp12::random(&mut rng);
        let c = &a - &b;

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let b_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc b"), &b);
        assert!(b_alloc.is_ok());
        let b_alloc = b_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.sub(&mut cs.namespace(|| "a-b"), &b_alloc);
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a-b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 3144);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_mul() {
        use std::ops::Mul;

        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let b = BlsFp12::random(&mut rng);
        let c = &a.clone().mul(&b);

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let b_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc b"), &b);
        assert!(b_alloc.is_ok());
        let b_alloc = b_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.mul(&mut cs.namespace(|| "a*b"), &b_alloc);
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a*b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 8715);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_square() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let c = &a.square();

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.square(&mut cs.namespace(|| "a²"));
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a² = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 8634);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_div() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let mut b = BlsFp12::random(&mut rng);
        while b.invert().is_none().into() {
            b = BlsFp12::random(&mut rng);
        }
        let c = &a * &b.invert().unwrap();

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let b_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc b"), &b);
        assert!(b_alloc.is_ok());
        let b_alloc = b_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.div_unchecked(&mut cs.namespace(|| "a div b"), &b_alloc);
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a div b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 11859);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_mul_by_014() {
        use bls12_381::fp2::Fp2 as BlsFp2;

        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let c0 = BlsFp2::random(&mut rng);
        let c1 = BlsFp2::random(&mut rng);
        let b = BlsFp12 {
            c0: BlsFp6 {
                c0,
                c1,
                c2: BlsFp2::zero(),
            },
            c1: BlsFp6 {
                c0: BlsFp2::zero(),
                c1: BlsFp2::one(),
                c2: BlsFp2::zero(),
            },
        };
        let c = &a * &b;

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let c0_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c0"), &c0);
        assert!(c0_alloc.is_ok());
        let c0_alloc = c0_alloc.unwrap();

        let c1_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c1"), &c1);
        assert!(c1_alloc.is_ok());
        let c1_alloc = c1_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.mul_by_014(
            &mut cs.namespace(|| "a*(c0, c1)(014)"),
            &c0_alloc,
            &c1_alloc,
        );
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a*b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 8430);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_mul_014_by_014() {
        use bls12_381::fp2::Fp2 as BlsFp2;

        let mut rng = rand::thread_rng();
        let c0 = BlsFp2::random(&mut rng);
        let c1 = BlsFp2::random(&mut rng);
        let d0 = BlsFp2::random(&mut rng);
        let d1 = BlsFp2::random(&mut rng);
        let a = BlsFp12 {
            c0: BlsFp6 {
                c0,
                c1,
                c2: BlsFp2::zero(),
            },
            c1: BlsFp6 {
                c0: BlsFp2::zero(),
                c1: BlsFp2::one(),
                c2: BlsFp2::zero(),
            },
        };
        let b = BlsFp12 {
            c0: BlsFp6 {
                c0: d0,
                c1: d1,
                c2: BlsFp2::zero(),
            },
            c1: BlsFp6 {
                c0: BlsFp2::zero(),
                c1: BlsFp2::one(),
                c2: BlsFp2::zero(),
            },
        };
        let c = &a * &b;

        let mut cs = TestConstraintSystem::<Fp>::new();

        let c0_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c0"), &c0);
        assert!(c0_alloc.is_ok());
        let c0_alloc = c0_alloc.unwrap();

        let c1_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c1"), &c1);
        assert!(c1_alloc.is_ok());
        let c1_alloc = c1_alloc.unwrap();

        let d0_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc d0"), &d0);
        assert!(d0_alloc.is_ok());
        let d0_alloc = d0_alloc.unwrap();

        let d1_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc d1"), &d1);
        assert!(d1_alloc.is_ok());
        let d1_alloc = d1_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = AllocatedE12Element::mul_014_by_014(
            &mut cs.namespace(|| "(c0, c1)(014)*(d0, d1)(014)"),
            &c0_alloc,
            &c1_alloc,
            &d0_alloc,
            &d1_alloc,
        );
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a*b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 7341);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_mul_by_01245() {
        use bls12_381::fp2::Fp2 as BlsFp2;

        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let c00 = BlsFp2::random(&mut rng);
        let c01 = BlsFp2::random(&mut rng);
        let c02 = BlsFp2::random(&mut rng);
        let c11 = BlsFp2::random(&mut rng);
        let c12 = BlsFp2::random(&mut rng);
        let b = BlsFp12 {
            c0: BlsFp6 {
                c0: c00,
                c1: c01,
                c2: c02,
            },
            c1: BlsFp6 {
                c0: BlsFp2::zero(),
                c1: c11,
                c2: c12,
            },
        };
        let c = &a * &b;

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let c00_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c00"), &c00);
        assert!(c00_alloc.is_ok());
        let c00_alloc = c00_alloc.unwrap();

        let c01_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c01"), &c01);
        assert!(c01_alloc.is_ok());
        let c01_alloc = c01_alloc.unwrap();

        let c02_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c02"), &c02);
        assert!(c02_alloc.is_ok());
        let c02_alloc = c02_alloc.unwrap();

        let c11_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c11"), &c11);
        assert!(c11_alloc.is_ok());
        let c11_alloc = c11_alloc.unwrap();

        let c12_alloc = AllocatedE2Element::alloc_element(&mut cs.namespace(|| "alloc c12"), &c12);
        assert!(c12_alloc.is_ok());
        let c12_alloc = c12_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.mul_by_01245(
            &mut cs.namespace(|| "a*(c00, c01, c01, c11, c12)(01245)"),
            &c00_alloc,
            &c01_alloc,
            &c02_alloc,
            &c11_alloc,
            &c12_alloc,
        );
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a*b = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 8676);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_conjugate() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let c = a.conjugate();

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.conjugate(&mut cs.namespace(|| "a.conjugate()"));
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a.conjugate() = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 3144);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_inverse() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let c = &a.invert().unwrap_or_else(|| BlsFp12::zero());

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let res_alloc = a_alloc.inverse(&mut cs.namespace(|| "a^-1"));
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a^-1 = c"),
            &res_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 11859);
        assert_eq!(cs.num_inputs(), 1);
    }

    #[test]
    fn test_random_alloc_is_zero() {
        let mut rng = rand::thread_rng();
        let a = BlsFp12::random(&mut rng);
        let b = BlsFp12::random(&mut rng);
        let c = b.clone();
        let zero = BlsFp12::zero();

        let mut cs = TestConstraintSystem::<Fp>::new();

        let a_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a"), &a);
        assert!(a_alloc.is_ok());
        let a_alloc = a_alloc.unwrap();

        let a2_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc a2"), &a);
        assert!(a2_alloc.is_ok());
        let a2_alloc = a2_alloc.unwrap();

        let b_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc b"), &b);
        assert!(b_alloc.is_ok());
        let b_alloc = b_alloc.unwrap();

        let res_alloc = a_alloc.sub(&mut cs.namespace(|| "a-a"), &a2_alloc);
        assert!(res_alloc.is_ok());
        let res_alloc = res_alloc.unwrap();

        let c_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc c"), &c);
        assert!(c_alloc.is_ok());
        let c_alloc = c_alloc.unwrap();

        let z_alloc = AllocatedE12Element::alloc_element(&mut cs.namespace(|| "alloc zero"), &zero);
        assert!(z_alloc.is_ok());
        let z_alloc = z_alloc.unwrap();

        let z2_alloc = z_alloc.square(&mut cs.namespace(|| "z2 <- z^2")).unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "a-a = z"),
            &res_alloc,
            &z2_alloc,
        );
        assert!(eq_alloc.is_ok());

        let zbit_alloc = res_alloc.alloc_is_zero(&mut cs.namespace(|| "z <- a-a ?= 0"));
        assert!(zbit_alloc.is_ok());
        let zbit_alloc = zbit_alloc.unwrap();

        let cond_alloc = AllocatedE12Element::conditionally_select(
            &mut cs.namespace(|| "select(a, b, z)"),
            &a_alloc,
            &b_alloc,
            &Boolean::Is(zbit_alloc),
        );
        assert!(cond_alloc.is_ok());
        let cond_alloc = cond_alloc.unwrap();

        let eq_alloc = AllocatedE12Element::assert_is_equal(
            &mut cs.namespace(|| "select(a, b, z) = c = b"),
            &cond_alloc,
            &c_alloc,
        );
        assert!(eq_alloc.is_ok());

        if !cs.is_satisfied() {
            eprintln!("{:?}", cs.which_is_unsatisfied())
        }
        assert!(cs.is_satisfied());
        assert_eq!(cs.num_constraints(), 19925);
        assert_eq!(cs.num_inputs(), 1);
    }
}
