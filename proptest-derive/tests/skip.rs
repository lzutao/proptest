// Copyright 2018 The proptest developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(never_type)]
#![allow(dead_code, unreachable_code)]

#[macro_use]
extern crate proptest_derive;
#[macro_use]
extern crate proptest;
use proptest::prelude::Arbitrary;

#[derive(Debug, Arbitrary, PartialEq)]
enum Ty1 {
    V1,
    V2(!),
    #[proptest(skip)]
    V3,
}

#[derive(Debug, Arbitrary, PartialEq)]
enum Ty2 {
    V1,
    V2,
    #[proptest(skip)]
    V3,
    #[proptest(skip)]
    V4,
}

proptest! {
    #[test]
    fn ty1_always_v1(v: Ty1) {
        prop_assert_eq!(v, Ty1::V1);
    }

    #[test]
    fn ty_always_1_or_2(v: Ty2) {
        prop_assert!(v == Ty2::V1 || v == Ty2::V2);
    }
}

#[test]
fn asserting_arbitrary() {
    fn assert_arbitrary<T: Arbitrary>() {}

    assert_arbitrary::<Ty1>();
    assert_arbitrary::<Ty2>();
}
