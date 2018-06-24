//-
// Copyright 2017 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Strategies for generating `std::Option` values.

#![cfg_attr(feature="cargo-clippy", allow(expl_impl_clone_on_copy))]

use core::fmt;
use core::marker::PhantomData;

use strategy::*;
use test_runner::*;

//==============================================================================
// Probability
//==============================================================================

/// Creates a `Probability` from some value that is convertible into it.
///
/// # Panics
///
/// Panics if the converted to probability would lie
/// outside interval `[0.0, 1.0]`. Consult the `Into` (or `From`)
/// implementations for more details.
pub fn prob(from: impl Into<Probability>) -> Probability {
    from.into()
}

impl Default for Probability {
    /// The default probability is 0.5, or 50% chance.
    fn default() -> Self { prob(0.5) }
}

impl From<f64> for Probability {
    /// Creates a `Probability` from a `f64`.
    ///
    /// # Panics
    ///
    /// Panics if the probability is outside interval `[0.0, 1.0]`.
    fn from(prob: f64) -> Self {
        Probability::new(prob)
    }
}

impl Probability {
    /// Creates a `Probability` from a `f64`.
    ///
    /// # Panics
    ///
    /// Panics if the probability is outside interval `[0.0, 1.0]`.
    pub fn new(prob: f64) -> Self {
        assert!(prob >= 0.0 && prob <= 1.0);
        Probability(prob)
    }

    // Don't rely on these existing internally:

    /// Merges self together with some other argument producing a product
    /// type expected by some impelementations of `A: Arbitrary` in
    /// `A::Parameters`. This can be more ergonomic to work with and may
    /// help type inference.
    pub fn with<X>(self, and: X) -> product_type![Self, X] {
        product_pack![self, and]
    }

    /// Merges self together with some other argument generated with a
    /// default value producing a product type expected by some
    /// impelementations of `A: Arbitrary` in `A::Parameters`.
    /// This can be more ergonomic to work with and may help type inference.
    pub fn lift<X: Default>(self) -> product_type![Self, X] {
        self.with(Default::default())
    }
}

#[cfg(feature = "frunk")]
use frunk_core::generic::Generic;

#[cfg(feature = "frunk")]
impl Generic for Probability {
    type Repr = f64;

    /// Converts the `Probability` into an `f64`.
    fn into(self) -> Self::Repr { self.0 }

    /// Creates a `Probability` from a `f64`.
    ///
    /// # Panics
    ///
    /// Panics if the probability is outside interval `[0.0, 1.0]`.
    fn from(r: Self::Repr) -> Self { r.into() }
}

impl From<Probability> for f64 {
    fn from(p: Probability) -> Self { p.0 }
}

/// A probability in the range `[0.0, 1.0]` with a default of `0.5`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Probability(f64);

//==============================================================================
// Strategies for Option
//==============================================================================

mapfn! {
    [] fn WrapSome[<T : fmt::Debug>](t: T) -> Option<T> {
        Some(t)
    }
}

struct NoneStrategy<T>(PhantomData<T>);
impl<T> Clone for NoneStrategy<T> {
    fn clone(&self) -> Self { *self }
}
impl<T> Copy for NoneStrategy<T> { }
impl<T> fmt::Debug for NoneStrategy<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoneStrategy")
    }
}
impl<T : fmt::Debug> Strategy for NoneStrategy<T> {
    type Tree = Self;
    type Value = Option<T>;

    fn new_tree(&self, _: &mut TestRunner) -> NewTree<Self> {
        Ok(*self)
    }
}
impl<T : fmt::Debug> ValueTree for NoneStrategy<T> {
    type Value = Option<T>;

    fn current(&self) -> Option<T> { None }
    fn simplify(&mut self) -> bool { false }
    fn complicate(&mut self) -> bool { false }
}

opaque_strategy_wrapper! {
    /// Strategy which generates `Option` values whose inner `Some` values are
    /// generated by another strategy.
    ///
    /// Constructed by other functions in this module.
    #[derive(Clone)]
    pub struct OptionStrategy[<T>][where T : Strategy]
        (TupleUnion<(W<NoneStrategy<T::Value>>,
                     W<statics::Map<T, WrapSome>>)>)
        -> OptionValueTree<T::Tree>;
    /// `ValueTree` type corresponding to `OptionStrategy`.
    #[derive(Clone, Debug)]
    pub struct OptionValueTree[<T>][where T : ValueTree]
        (TupleUnionValueTree<(NoneStrategy<T::Value>,
                              Option<statics::Map<T, WrapSome>>)>)
        -> Option<T::Value>;
}

// XXX Unclear why this is necessary; #[derive(Debug)] *should* generate
// exactly this, but for some reason it adds a `T::Value : Debug` constraint as
// well.
impl<T : Strategy + fmt::Debug> fmt::Debug for OptionStrategy<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OptionStrategy({:?})", self.0)
    }
}

/// Return a strategy producing `Optional` values wrapping values from the
/// given delegate strategy.
///
/// `Some` values shrink to `None`.
///
/// `Some` and `None` are each chosen with 50% probability.
pub fn of<T : Strategy>(t: T) -> OptionStrategy<T> {
    weighted(Probability::default(), t)
}

/// Return a strategy producing `Optional` values wrapping values from the
/// given delegate strategy.
///
/// `Some` values shrink to `None`.
///
/// `Some` is chosen with a probability given by `probability_of_some`, which
/// must be between 0.0 and 1.0, both exclusive.
pub fn weighted<T : Strategy>
    (probability_of_some: impl Into<Probability>, t: T) -> OptionStrategy<T>
{
    let prob = probability_of_some.into().into();
    let (weight_some, weight_none) = float_to_weight(prob);

    OptionStrategy(TupleUnion::new((
        (weight_none, NoneStrategy(PhantomData)),
        (weight_some, statics::Map::new(t, WrapSome)),
    )))
}

#[cfg(test)]
mod test {
    use super::*;

    fn count_some_of_1000(s: OptionStrategy<Just<i32>>) -> u32 {
        let mut runner = TestRunner::default();
        let mut count = 0;
        for _ in 0..1000 {
            count += s.new_tree(&mut runner).unwrap()
                .current().is_some() as u32;
        }

        count
    }

    #[test]
    fn probability_defaults_to_0p5() {
        let count = count_some_of_1000(of(Just(42i32)));
        assert!(count > 450 && count < 550);
    }

    #[test]
    fn probability_handled_correctly() {
        let count = count_some_of_1000(weighted(0.9, Just(42i32)));
        assert!(count > 800 && count < 950);

        let count = count_some_of_1000(weighted(0.1, Just(42i32)));
        assert!(count > 50 && count < 150);
    }

    #[test]
    fn test_sanity() {
        check_strategy_sanity(of(0i32..1000i32), None);
    }
}
