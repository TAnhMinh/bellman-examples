#![allow(unused_imports)]
#![allow(unused_variables)]
extern crate bellman;
extern crate pairing;
extern crate rand;

// Based on bellman-tutorial crate by Jay Graber
// created so that I can get my hands on creating a proof

// For randomness (during paramgen and proof generation)
use self::rand::{thread_rng, Rng};

// Bring in some tools for using pairing-friendly curves
use self::pairing::{
    Engine,
    Field,
    PrimeField
};

// We're going to use the BLS12-381 pairing-friendly elliptic curve.
use self::pairing::bls12_381::{
    Bls12,
    Fr
};

// We'll use these interfaces to construct our circuit.
use self::bellman::{
    Circuit,
    ConstraintSystem,
    SynthesisError
};

// We're going to use the Groth16 proving system.
use self::bellman::groth16::{
    Proof,
    generate_random_parameters,
    prepare_verifying_key,
    create_random_proof,
    verify_proof,
};

// proving that I know x such that x^4 - 10x^3 + 35x^2 - 50x + 24 = 0
// which is (x-1)(x-2)(x-3)(x-4)
// Generalized: x^4 - 10x^3 + 35x^2 - 50x + 24 = 0 == out
pub struct QuarticDemo<E: Engine> {
    pub x: Option<E::Fr>,
}

impl <E: Engine> Circuit<E> for QuarticDemo<E> {
    fn synthesize<CS: ConstraintSystem<E>>(
        self,
        cs: &mut CS
    ) -> Result<(), SynthesisError>
    {
        // Flattened into quartic equation x^4 - 10x^3 + 35x^2 - 50x + 24 == 0:
        // x * x = x_2
        // x_2 * x = x_3
        // x_3 * x = x_4
        // x_4 - 10*x_3 = tmp_1
        // tmp_1 + 35*x_2 = tmp_2
        // tmp_2 - 50*x = tmp_3
        // tmp_3 + 24 = out
        // Resulting R1CS with w = [one, x, x_2, x_3, x_4, tmp_1, tmp_2, tmp_3, out]

        // Allocate the first private "auxiliary" variable
        let x_val = self.x;
        let x = cs.alloc(|| "x", || {
            x_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate: x * x = x_2
        let x_2_val = x_val.map(|mut e| {
            e.square();
            e
        });
        let x_2 = cs.alloc(|| "x_2", || {
            x_2_val.ok_or(SynthesisError::AssignmentMissing)
        })?;
        // Enforce: x * x = x_2
        cs.enforce(
            || "x_2",
            |lc| lc + x,
            |lc| lc + x,
            |lc| lc + x_2
        );

        // Allocate: x_2 * x = x_3
        let x_3_val =x_2_val.map(|mut e| {
            e.mul_assign(&x_val.unwrap());
            e
        });
        let x_3 = cs.alloc(|| "x_3", || {
            x_3_val.ok_or(SynthesisError::AssignmentMissing)
        })?;
        // Enforce: x_2 * x = x_3
        cs.enforce(
            || "x_3",
            |lc| lc + x_2,
            |lc| lc + x,
            |lc| lc + x_3
        );

        // Allocate: x_3 * x = x_4
        let x_4_val = x_3_val.map(|mut e| {
            e.mul_assign(&x_val.unwrap());
            e
        });
        let x_4 = cs.alloc(|| "x_4", || {
            x_4_val.ok_or(SynthesisError::AssignmentMissing)
        })?;
        // Enforce: x_3 * x = x_4
        cs.enforce(
            || "x_4",
            |lc| lc + x_3,
            |lc| lc + x,
            |lc| lc + x_4
        );


        let x_3_10_val = x_3_val.map(|mut e| {
            e.mul_assign(&E::Fr::from_str("10").unwrap());
            e.negate();
            e
        });
        let x_3_10 = cs.alloc(|| "x_3_10", || {
            x_3_10_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let x_2_35_val = x_2_val.map(|mut e| {
            e.mul_assign(&E::Fr::from_str("35").unwrap());
            e
        });
        let x_2_35 = cs.alloc(|| "x_2_35", || {
            x_2_35_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let x_1_50_val = x_val.map(|mut e| {
            e.mul_assign(&E::Fr::from_str("50").unwrap());
            e.negate();//unwrap?
            e
        });
        let x_1_50 = cs.alloc(|| "x_1_50", || {
            x_1_50_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocating the public "primary" output uses alloc_input
        let out = cs.alloc_input(|| "out", || {
            let mut tmp = x_4_val.unwrap();
            tmp.add_assign(&x_3_10_val.unwrap());
            tmp.add_assign(&x_2_35_val.unwrap());
            tmp.add_assign(&x_1_50_val.unwrap());
            tmp.add_assign(&E::Fr::from_str("24").unwrap());
            Ok(tmp)
        })?;

        // tmp_3 + 24 = out
        // => (tmp_3 + 24) * 1 = out
        cs.enforce(
            || "out",
            |lc| lc + x_4 + x_3_10 + x_2_35 + x_1_50 + (E::Fr::from_str("24").unwrap(), CS::one()), //co znamena tento radek
            |lc| lc + CS::one(),
            |lc| lc + out
        );
        // lc is an inner product of all variables with some vector of coefficients
        // bunch of variables added together with some coefficients

        // usually if mult by 1 can do more efficiently
        // x2 * x = out - x - 5

        // mult quadratic constraints
        //

        Ok(())
    }
}
