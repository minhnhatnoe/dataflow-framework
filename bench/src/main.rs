#![allow(non_snake_case, unused_imports, dead_code)]
use std::hint::black_box;
use std::time::Instant;

mod flat {
use dataflow_flat::{Component, ComponentSubscription, Root};
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Root)]
pub struct Poll;
pub static OUT: AtomicU64 = AtomicU64::new(0);
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_0(pub u64);
impl Component for C0_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_1(pub u64);
impl Component for C0_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(2); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_2(pub u64);
impl Component for C0_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(3); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_3(pub u64);
impl Component for C0_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(4); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_4(pub u64);
impl Component for C0_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(5); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_5(pub u64);
impl Component for C0_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(6); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_6(pub u64);
impl Component for C0_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(7); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_7(pub u64);
impl Component for C0_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(8); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_8(pub u64);
impl Component for C0_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(9); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_9(pub u64);
impl Component for C0_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(10); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_10(pub u64);
impl Component for C0_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(11); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_11(pub u64);
impl Component for C0_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(12); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_12(pub u64);
impl Component for C0_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(13); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_13(pub u64);
impl Component for C0_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(14); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_14(pub u64);
impl Component for C0_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(15); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_15(pub u64);
impl Component for C0_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(16); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_16(pub u64);
impl Component for C0_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(17); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_17(pub u64);
impl Component for C0_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(18); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_18(pub u64);
impl Component for C0_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(19); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct C0_19(pub u64);
impl Component for C0_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C0_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(20); self.0 }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_4, s1: C0_18, s2: C0_2, s3: C0_8)]
pub struct C1_0(pub u64);
impl Component for C1_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(31) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_3, s1: C0_15, s2: C0_14, s3: C0_18)]
pub struct C1_1(pub u64);
impl Component for C1_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(32) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_12, s1: C0_6, s2: C0_3, s3: C0_15)]
pub struct C1_2(pub u64);
impl Component for C1_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(33) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_0, s1: C0_12, s2: C0_13, s3: C0_19)]
pub struct C1_3(pub u64);
impl Component for C1_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(34) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_14, s1: C0_8, s2: C0_7, s3: C0_3)]
pub struct C1_4(pub u64);
impl Component for C1_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(35) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_10, s1: C0_0, s2: C0_18, s3: C0_17)]
pub struct C1_5(pub u64);
impl Component for C1_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(36) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_17, s1: C0_0, s2: C0_12, s3: C0_6)]
pub struct C1_6(pub u64);
impl Component for C1_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(37) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_13, s1: C0_0, s2: C0_16, s3: C0_7)]
pub struct C1_7(pub u64);
impl Component for C1_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(38) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_14, s1: C0_15, s2: C0_17, s3: C0_7)]
pub struct C1_8(pub u64);
impl Component for C1_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(39) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_11, s1: C0_7, s2: C0_18, s3: C0_14)]
pub struct C1_9(pub u64);
impl Component for C1_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(40) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_9, s1: C0_0, s2: C0_13, s3: C0_3)]
pub struct C1_10(pub u64);
impl Component for C1_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(41) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_5, s1: C0_9, s2: C0_3, s3: C0_10)]
pub struct C1_11(pub u64);
impl Component for C1_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(42) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_16, s1: C0_13, s2: C0_19, s3: C0_6)]
pub struct C1_12(pub u64);
impl Component for C1_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(43) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_9, s1: C0_19, s2: C0_15, s3: C0_16)]
pub struct C1_13(pub u64);
impl Component for C1_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(44) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_12, s1: C0_18, s2: C0_1, s3: C0_15)]
pub struct C1_14(pub u64);
impl Component for C1_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(45) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_7, s1: C0_12, s2: C0_13, s3: C0_5)]
pub struct C1_15(pub u64);
impl Component for C1_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(46) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_11, s1: C0_17, s2: C0_19, s3: C0_2)]
pub struct C1_16(pub u64);
impl Component for C1_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(47) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_14, s1: C0_16, s2: C0_3, s3: C0_5)]
pub struct C1_17(pub u64);
impl Component for C1_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(48) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_16, s1: C0_12, s2: C0_11, s3: C0_15)]
pub struct C1_18(pub u64);
impl Component for C1_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(49) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C0_0, s1: C0_15, s2: C0_1, s3: C0_9)]
pub struct C1_19(pub u64);
impl Component for C1_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C1_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(50) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_19, s1: C1_18, s2: C1_12, s3: C1_5)]
pub struct C2_0(pub u64);
impl Component for C2_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(62) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_5, s1: C1_16, s2: C1_7, s3: C1_0)]
pub struct C2_1(pub u64);
impl Component for C2_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(63) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_6, s1: C1_17, s2: C1_18, s3: C1_7)]
pub struct C2_2(pub u64);
impl Component for C2_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(64) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_12, s1: C1_16, s2: C1_11, s3: C1_17)]
pub struct C2_3(pub u64);
impl Component for C2_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(65) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_14, s1: C1_8, s2: C1_17, s3: C1_0)]
pub struct C2_4(pub u64);
impl Component for C2_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(66) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_12, s1: C1_16, s2: C1_4, s3: C1_18)]
pub struct C2_5(pub u64);
impl Component for C2_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(67) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_17, s1: C1_6, s2: C1_13, s3: C1_1)]
pub struct C2_6(pub u64);
impl Component for C2_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(68) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_15, s1: C1_11, s2: C1_17, s3: C1_6)]
pub struct C2_7(pub u64);
impl Component for C2_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(69) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_16, s1: C1_13, s2: C1_15, s3: C1_11)]
pub struct C2_8(pub u64);
impl Component for C2_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(70) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_13, s1: C1_11, s2: C1_0, s3: C1_10)]
pub struct C2_9(pub u64);
impl Component for C2_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(71) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_14, s1: C1_0, s2: C1_7, s3: C1_5)]
pub struct C2_10(pub u64);
impl Component for C2_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(72) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_17, s1: C1_18, s2: C1_5, s3: C1_2)]
pub struct C2_11(pub u64);
impl Component for C2_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(73) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_17, s1: C1_8, s2: C1_1, s3: C1_2)]
pub struct C2_12(pub u64);
impl Component for C2_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(74) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_2, s1: C1_0, s2: C1_14, s3: C1_18)]
pub struct C2_13(pub u64);
impl Component for C2_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(75) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_8, s1: C1_7, s2: C1_19, s3: C1_3)]
pub struct C2_14(pub u64);
impl Component for C2_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(76) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_19, s1: C1_5, s2: C1_11, s3: C1_9)]
pub struct C2_15(pub u64);
impl Component for C2_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(77) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_2, s1: C1_5, s2: C1_18, s3: C1_8)]
pub struct C2_16(pub u64);
impl Component for C2_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(78) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_16, s1: C1_5, s2: C1_8, s3: C1_9)]
pub struct C2_17(pub u64);
impl Component for C2_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(79) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_14, s1: C1_10, s2: C1_15, s3: C1_17)]
pub struct C2_18(pub u64);
impl Component for C2_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(80) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C1_3, s1: C1_0, s2: C1_9, s3: C1_12)]
pub struct C2_19(pub u64);
impl Component for C2_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C2_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(81) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_10, s1: C2_13, s2: C2_6, s3: C2_8)]
pub struct C3_0(pub u64);
impl Component for C3_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(93) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_3, s1: C2_8, s2: C2_16, s3: C2_6)]
pub struct C3_1(pub u64);
impl Component for C3_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(94) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_19, s1: C2_13, s2: C2_0, s3: C2_7)]
pub struct C3_2(pub u64);
impl Component for C3_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(95) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_0, s1: C2_12, s2: C2_4, s3: C2_1)]
pub struct C3_3(pub u64);
impl Component for C3_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(96) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_5, s1: C2_14, s2: C2_16, s3: C2_13)]
pub struct C3_4(pub u64);
impl Component for C3_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(97) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_17, s1: C2_7, s2: C2_16, s3: C2_14)]
pub struct C3_5(pub u64);
impl Component for C3_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(98) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_7, s1: C2_16, s2: C2_0, s3: C2_12)]
pub struct C3_6(pub u64);
impl Component for C3_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(99) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_18, s1: C2_10, s2: C2_13, s3: C2_1)]
pub struct C3_7(pub u64);
impl Component for C3_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(100) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_9, s1: C2_4, s2: C2_6, s3: C2_1)]
pub struct C3_8(pub u64);
impl Component for C3_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(101) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_9, s1: C2_2, s2: C2_18, s3: C2_19)]
pub struct C3_9(pub u64);
impl Component for C3_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(102) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_9, s1: C2_5, s2: C2_13, s3: C2_8)]
pub struct C3_10(pub u64);
impl Component for C3_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(103) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_4, s1: C2_0, s2: C2_17, s3: C2_1)]
pub struct C3_11(pub u64);
impl Component for C3_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(104) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_18, s1: C2_6, s2: C2_14, s3: C2_5)]
pub struct C3_12(pub u64);
impl Component for C3_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(105) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_19, s1: C2_16, s2: C2_1, s3: C2_12)]
pub struct C3_13(pub u64);
impl Component for C3_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(106) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_6, s1: C2_11, s2: C2_3, s3: C2_19)]
pub struct C3_14(pub u64);
impl Component for C3_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(107) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_18, s1: C2_13, s2: C2_6, s3: C2_15)]
pub struct C3_15(pub u64);
impl Component for C3_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(108) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_3, s1: C2_12, s2: C2_9, s3: C2_16)]
pub struct C3_16(pub u64);
impl Component for C3_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(109) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_15, s1: C2_0, s2: C2_10, s3: C2_12)]
pub struct C3_17(pub u64);
impl Component for C3_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(110) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_9, s1: C2_0, s2: C2_5, s3: C2_6)]
pub struct C3_18(pub u64);
impl Component for C3_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(111) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C2_10, s1: C2_18, s2: C2_4, s3: C2_19)]
pub struct C3_19(pub u64);
impl Component for C3_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C3_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(112) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_13, s1: C3_6, s2: C3_8, s3: C3_3)]
pub struct C4_0(pub u64);
impl Component for C4_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(124) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_12, s1: C3_17, s2: C3_11, s3: C3_15)]
pub struct C4_1(pub u64);
impl Component for C4_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(125) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_17, s1: C3_7, s2: C3_2, s3: C3_1)]
pub struct C4_2(pub u64);
impl Component for C4_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(126) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_2, s1: C3_4, s2: C3_5, s3: C3_17)]
pub struct C4_3(pub u64);
impl Component for C4_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(127) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_17, s1: C3_6, s2: C3_8, s3: C3_10)]
pub struct C4_4(pub u64);
impl Component for C4_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(128) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_19, s1: C3_16, s2: C3_8, s3: C3_11)]
pub struct C4_5(pub u64);
impl Component for C4_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(129) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_10, s1: C3_19, s2: C3_3, s3: C3_9)]
pub struct C4_6(pub u64);
impl Component for C4_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(130) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_7, s1: C3_15, s2: C3_4, s3: C3_3)]
pub struct C4_7(pub u64);
impl Component for C4_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(131) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_10, s1: C3_1, s2: C3_13, s3: C3_2)]
pub struct C4_8(pub u64);
impl Component for C4_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(132) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_12, s1: C3_4, s2: C3_18, s3: C3_10)]
pub struct C4_9(pub u64);
impl Component for C4_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(133) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_3, s1: C3_18, s2: C3_12, s3: C3_2)]
pub struct C4_10(pub u64);
impl Component for C4_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(134) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_18, s1: C3_17, s2: C3_7, s3: C3_2)]
pub struct C4_11(pub u64);
impl Component for C4_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(135) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_8, s1: C3_11, s2: C3_9, s3: C3_3)]
pub struct C4_12(pub u64);
impl Component for C4_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(136) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_14, s1: C3_8, s2: C3_3, s3: C3_1)]
pub struct C4_13(pub u64);
impl Component for C4_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(137) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_9, s1: C3_0, s2: C3_18, s3: C3_2)]
pub struct C4_14(pub u64);
impl Component for C4_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(138) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_13, s1: C3_3, s2: C3_1, s3: C3_6)]
pub struct C4_15(pub u64);
impl Component for C4_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(139) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_7, s1: C3_18, s2: C3_13, s3: C3_5)]
pub struct C4_16(pub u64);
impl Component for C4_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(140) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_3, s1: C3_14, s2: C3_5, s3: C3_7)]
pub struct C4_17(pub u64);
impl Component for C4_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(141) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_5, s1: C3_3, s2: C3_13, s3: C3_12)]
pub struct C4_18(pub u64);
impl Component for C4_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(142) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C3_17, s1: C3_9, s2: C3_19, s3: C3_8)]
pub struct C4_19(pub u64);
impl Component for C4_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C4_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(143) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_15, s1: C4_10, s2: C4_3, s3: C4_6)]
pub struct C5_0(pub u64);
impl Component for C5_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(155) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_10, s1: C4_1, s2: C4_0, s3: C4_17)]
pub struct C5_1(pub u64);
impl Component for C5_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(156) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_9, s1: C4_10, s2: C4_14, s3: C4_12)]
pub struct C5_2(pub u64);
impl Component for C5_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(157) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_10, s1: C4_12, s2: C4_2, s3: C4_17)]
pub struct C5_3(pub u64);
impl Component for C5_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(158) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_10, s1: C4_14, s2: C4_3, s3: C4_8)]
pub struct C5_4(pub u64);
impl Component for C5_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(159) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_6, s1: C4_17, s2: C4_15, s3: C4_11)]
pub struct C5_5(pub u64);
impl Component for C5_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(160) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_8, s1: C4_5, s2: C4_17, s3: C4_6)]
pub struct C5_6(pub u64);
impl Component for C5_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(161) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_9, s1: C4_6, s2: C4_7, s3: C4_11)]
pub struct C5_7(pub u64);
impl Component for C5_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(162) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_2, s1: C4_8, s2: C4_19, s3: C4_14)]
pub struct C5_8(pub u64);
impl Component for C5_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(163) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_2, s1: C4_18, s2: C4_10, s3: C4_7)]
pub struct C5_9(pub u64);
impl Component for C5_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(164) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_12, s1: C4_9, s2: C4_1, s3: C4_10)]
pub struct C5_10(pub u64);
impl Component for C5_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(165) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_5, s1: C4_10, s2: C4_9, s3: C4_7)]
pub struct C5_11(pub u64);
impl Component for C5_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(166) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_10, s1: C4_3, s2: C4_17, s3: C4_2)]
pub struct C5_12(pub u64);
impl Component for C5_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(167) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_7, s1: C4_19, s2: C4_0, s3: C4_18)]
pub struct C5_13(pub u64);
impl Component for C5_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(168) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_12, s1: C4_2, s2: C4_8, s3: C4_18)]
pub struct C5_14(pub u64);
impl Component for C5_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(169) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_2, s1: C4_0, s2: C4_18, s3: C4_9)]
pub struct C5_15(pub u64);
impl Component for C5_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(170) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_11, s1: C4_15, s2: C4_18, s3: C4_4)]
pub struct C5_16(pub u64);
impl Component for C5_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(171) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_3, s1: C4_16, s2: C4_10, s3: C4_2)]
pub struct C5_17(pub u64);
impl Component for C5_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(172) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_16, s1: C4_5, s2: C4_18, s3: C4_4)]
pub struct C5_18(pub u64);
impl Component for C5_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(173) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C4_4, s1: C4_10, s2: C4_9, s3: C4_3)]
pub struct C5_19(pub u64);
impl Component for C5_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C5_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(174) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_16, s1: C5_9, s2: C5_4, s3: C5_6)]
pub struct C6_0(pub u64);
impl Component for C6_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(186) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_4, s1: C5_17, s2: C5_1, s3: C5_10)]
pub struct C6_1(pub u64);
impl Component for C6_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(187) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_19, s1: C5_17, s2: C5_6, s3: C5_5)]
pub struct C6_2(pub u64);
impl Component for C6_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(188) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_9, s1: C5_13, s2: C5_17, s3: C5_5)]
pub struct C6_3(pub u64);
impl Component for C6_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(189) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_1, s1: C5_7, s2: C5_8, s3: C5_2)]
pub struct C6_4(pub u64);
impl Component for C6_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(190) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_14, s1: C5_13, s2: C5_17, s3: C5_8)]
pub struct C6_5(pub u64);
impl Component for C6_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(191) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_17, s1: C5_14, s2: C5_19, s3: C5_18)]
pub struct C6_6(pub u64);
impl Component for C6_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(192) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_0, s1: C5_12, s2: C5_10, s3: C5_5)]
pub struct C6_7(pub u64);
impl Component for C6_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(193) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_8, s1: C5_15, s2: C5_0, s3: C5_13)]
pub struct C6_8(pub u64);
impl Component for C6_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(194) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_18, s1: C5_0, s2: C5_1, s3: C5_11)]
pub struct C6_9(pub u64);
impl Component for C6_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(195) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_18, s1: C5_4, s2: C5_19, s3: C5_17)]
pub struct C6_10(pub u64);
impl Component for C6_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(196) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_8, s1: C5_19, s2: C5_12, s3: C5_17)]
pub struct C6_11(pub u64);
impl Component for C6_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(197) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_5, s1: C5_2, s2: C5_7, s3: C5_15)]
pub struct C6_12(pub u64);
impl Component for C6_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(198) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_0, s1: C5_5, s2: C5_16, s3: C5_10)]
pub struct C6_13(pub u64);
impl Component for C6_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(199) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_16, s1: C5_14, s2: C5_7, s3: C5_17)]
pub struct C6_14(pub u64);
impl Component for C6_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(200) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_10, s1: C5_15, s2: C5_18, s3: C5_7)]
pub struct C6_15(pub u64);
impl Component for C6_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(201) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_13, s1: C5_10, s2: C5_17, s3: C5_8)]
pub struct C6_16(pub u64);
impl Component for C6_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(202) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_7, s1: C5_1, s2: C5_2, s3: C5_16)]
pub struct C6_17(pub u64);
impl Component for C6_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(203) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_11, s1: C5_5, s2: C5_16, s3: C5_6)]
pub struct C6_18(pub u64);
impl Component for C6_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(204) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C5_9, s1: C5_19, s2: C5_18, s3: C5_11)]
pub struct C6_19(pub u64);
impl Component for C6_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C6_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(205) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_5, s1: C6_14, s2: C6_2, s3: C6_3)]
pub struct C7_0(pub u64);
impl Component for C7_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(217) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_19, s1: C6_16, s2: C6_12, s3: C6_5)]
pub struct C7_1(pub u64);
impl Component for C7_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(218) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_4, s1: C6_8, s2: C6_13, s3: C6_6)]
pub struct C7_2(pub u64);
impl Component for C7_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(219) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_18, s1: C6_1, s2: C6_15, s3: C6_12)]
pub struct C7_3(pub u64);
impl Component for C7_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(220) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_11, s1: C6_12, s2: C6_16, s3: C6_5)]
pub struct C7_4(pub u64);
impl Component for C7_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(221) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_17, s1: C6_1, s2: C6_16, s3: C6_2)]
pub struct C7_5(pub u64);
impl Component for C7_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(222) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_8, s1: C6_3, s2: C6_19, s3: C6_2)]
pub struct C7_6(pub u64);
impl Component for C7_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(223) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_4, s1: C6_2, s2: C6_14, s3: C6_7)]
pub struct C7_7(pub u64);
impl Component for C7_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(224) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_12, s1: C6_13, s2: C6_19, s3: C6_5)]
pub struct C7_8(pub u64);
impl Component for C7_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(225) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_10, s1: C6_14, s2: C6_4, s3: C6_15)]
pub struct C7_9(pub u64);
impl Component for C7_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(226) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_6, s1: C6_3, s2: C6_13, s3: C6_17)]
pub struct C7_10(pub u64);
impl Component for C7_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(227) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_3, s1: C6_9, s2: C6_8, s3: C6_7)]
pub struct C7_11(pub u64);
impl Component for C7_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(228) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_12, s1: C6_17, s2: C6_0, s3: C6_6)]
pub struct C7_12(pub u64);
impl Component for C7_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(229) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_16, s1: C6_14, s2: C6_0, s3: C6_17)]
pub struct C7_13(pub u64);
impl Component for C7_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(230) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_19, s1: C6_7, s2: C6_8, s3: C6_6)]
pub struct C7_14(pub u64);
impl Component for C7_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(231) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_5, s1: C6_9, s2: C6_4, s3: C6_6)]
pub struct C7_15(pub u64);
impl Component for C7_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(232) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_8, s1: C6_9, s2: C6_19, s3: C6_14)]
pub struct C7_16(pub u64);
impl Component for C7_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(233) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_5, s1: C6_17, s2: C6_11, s3: C6_15)]
pub struct C7_17(pub u64);
impl Component for C7_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(234) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_13, s1: C6_3, s2: C6_6, s3: C6_12)]
pub struct C7_18(pub u64);
impl Component for C7_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(235) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C6_6, s1: C6_9, s2: C6_3, s3: C6_0)]
pub struct C7_19(pub u64);
impl Component for C7_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C7_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(236) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_3, s1: C7_18, s2: C7_0, s3: C7_9)]
pub struct C8_0(pub u64);
impl Component for C8_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(248) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_4, s1: C7_2, s2: C7_16, s3: C7_11)]
pub struct C8_1(pub u64);
impl Component for C8_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(249) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_18, s1: C7_9, s2: C7_13, s3: C7_16)]
pub struct C8_2(pub u64);
impl Component for C8_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(250) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_11, s1: C7_16, s2: C7_10, s3: C7_0)]
pub struct C8_3(pub u64);
impl Component for C8_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(251) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_3, s1: C7_14, s2: C7_18, s3: C7_11)]
pub struct C8_4(pub u64);
impl Component for C8_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(252) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_9, s1: C7_17, s2: C7_12, s3: C7_10)]
pub struct C8_5(pub u64);
impl Component for C8_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(253) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_18, s1: C7_15, s2: C7_3, s3: C7_12)]
pub struct C8_6(pub u64);
impl Component for C8_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(254) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_12, s1: C7_6, s2: C7_17, s3: C7_0)]
pub struct C8_7(pub u64);
impl Component for C8_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(255) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_8, s1: C7_16, s2: C7_6, s3: C7_14)]
pub struct C8_8(pub u64);
impl Component for C8_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(256) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_19, s1: C7_16, s2: C7_13, s3: C7_9)]
pub struct C8_9(pub u64);
impl Component for C8_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(257) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_5, s1: C7_14, s2: C7_16, s3: C7_6)]
pub struct C8_10(pub u64);
impl Component for C8_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(258) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_11, s1: C7_16, s2: C7_0, s3: C7_12)]
pub struct C8_11(pub u64);
impl Component for C8_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(259) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_18, s1: C7_13, s2: C7_12, s3: C7_10)]
pub struct C8_12(pub u64);
impl Component for C8_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(260) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_19, s1: C7_18, s2: C7_2, s3: C7_15)]
pub struct C8_13(pub u64);
impl Component for C8_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(261) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_7, s1: C7_9, s2: C7_0, s3: C7_13)]
pub struct C8_14(pub u64);
impl Component for C8_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(262) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_4, s1: C7_12, s2: C7_8, s3: C7_5)]
pub struct C8_15(pub u64);
impl Component for C8_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(263) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_2, s1: C7_0, s2: C7_11, s3: C7_8)]
pub struct C8_16(pub u64);
impl Component for C8_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(264) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_13, s1: C7_17, s2: C7_9, s3: C7_4)]
pub struct C8_17(pub u64);
impl Component for C8_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(265) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_14, s1: C7_8, s2: C7_15, s3: C7_5)]
pub struct C8_18(pub u64);
impl Component for C8_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(266) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C7_14, s1: C7_16, s2: C7_1, s3: C7_8)]
pub struct C8_19(pub u64);
impl Component for C8_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C8_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(267) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_16, s1: C8_3, s2: C8_13, s3: C8_2)]
pub struct C9_0(pub u64);
impl Component for C9_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(279) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_11, s1: C8_2, s2: C8_14, s3: C8_0)]
pub struct C9_1(pub u64);
impl Component for C9_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(280) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_5, s1: C8_16, s2: C8_19, s3: C8_2)]
pub struct C9_2(pub u64);
impl Component for C9_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(281) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_12, s1: C8_8, s2: C8_9, s3: C8_6)]
pub struct C9_3(pub u64);
impl Component for C9_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(282) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_16, s1: C8_6, s2: C8_7, s3: C8_10)]
pub struct C9_4(pub u64);
impl Component for C9_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(283) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_8, s1: C8_2, s2: C8_18, s3: C8_16)]
pub struct C9_5(pub u64);
impl Component for C9_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(284) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_11, s1: C8_14, s2: C8_16, s3: C8_1)]
pub struct C9_6(pub u64);
impl Component for C9_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(285) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_5, s1: C8_9, s2: C8_17, s3: C8_8)]
pub struct C9_7(pub u64);
impl Component for C9_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(286) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_11, s1: C8_7, s2: C8_12, s3: C8_17)]
pub struct C9_8(pub u64);
impl Component for C9_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(287) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_5, s1: C8_15, s2: C8_8, s3: C8_10)]
pub struct C9_9(pub u64);
impl Component for C9_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(288) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_7, s1: C8_8, s2: C8_19, s3: C8_0)]
pub struct C9_10(pub u64);
impl Component for C9_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(289) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_19, s1: C8_12, s2: C8_10, s3: C8_13)]
pub struct C9_11(pub u64);
impl Component for C9_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(290) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_7, s1: C8_8, s2: C8_6, s3: C8_2)]
pub struct C9_12(pub u64);
impl Component for C9_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(291) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_5, s1: C8_18, s2: C8_14, s3: C8_4)]
pub struct C9_13(pub u64);
impl Component for C9_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(292) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_19, s1: C8_8, s2: C8_14, s3: C8_16)]
pub struct C9_14(pub u64);
impl Component for C9_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(293) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_5, s1: C8_4, s2: C8_18, s3: C8_14)]
pub struct C9_15(pub u64);
impl Component for C9_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(294) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_11, s1: C8_9, s2: C8_12, s3: C8_7)]
pub struct C9_16(pub u64);
impl Component for C9_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(295) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_3, s1: C8_6, s2: C8_9, s3: C8_2)]
pub struct C9_17(pub u64);
impl Component for C9_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(296) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_3, s1: C8_7, s2: C8_12, s3: C8_10)]
pub struct C9_18(pub u64);
impl Component for C9_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(297) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C8_15, s1: C8_3, s2: C8_5, s3: C8_1)]
pub struct C9_19(pub u64);
impl Component for C9_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C9_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(298) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_1, s1: C9_0, s2: C9_6, s3: C9_19)]
pub struct C10_0(pub u64);
impl Component for C10_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(310) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_15, s1: C9_16, s2: C9_14, s3: C9_10)]
pub struct C10_1(pub u64);
impl Component for C10_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(311) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_8, s1: C9_3, s2: C9_5, s3: C9_18)]
pub struct C10_2(pub u64);
impl Component for C10_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(312) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_7, s1: C9_12, s2: C9_19, s3: C9_15)]
pub struct C10_3(pub u64);
impl Component for C10_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(313) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_14, s1: C9_12, s2: C9_5, s3: C9_7)]
pub struct C10_4(pub u64);
impl Component for C10_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(314) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_7, s1: C9_9, s2: C9_14, s3: C9_12)]
pub struct C10_5(pub u64);
impl Component for C10_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(315) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_6, s1: C9_14, s2: C9_8, s3: C9_10)]
pub struct C10_6(pub u64);
impl Component for C10_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(316) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_15, s1: C9_18, s2: C9_3, s3: C9_6)]
pub struct C10_7(pub u64);
impl Component for C10_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(317) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_2, s1: C9_1, s2: C9_0, s3: C9_17)]
pub struct C10_8(pub u64);
impl Component for C10_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(318) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_15, s1: C9_10, s2: C9_12, s3: C9_9)]
pub struct C10_9(pub u64);
impl Component for C10_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(319) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_6, s1: C9_12, s2: C9_5, s3: C9_4)]
pub struct C10_10(pub u64);
impl Component for C10_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(320) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_0, s1: C9_19, s2: C9_12, s3: C9_4)]
pub struct C10_11(pub u64);
impl Component for C10_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(321) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_17, s1: C9_1, s2: C9_12, s3: C9_8)]
pub struct C10_12(pub u64);
impl Component for C10_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(322) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_4, s1: C9_2, s2: C9_14, s3: C9_9)]
pub struct C10_13(pub u64);
impl Component for C10_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(323) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_0, s1: C9_1, s2: C9_17, s3: C9_18)]
pub struct C10_14(pub u64);
impl Component for C10_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(324) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_16, s1: C9_4, s2: C9_1, s3: C9_8)]
pub struct C10_15(pub u64);
impl Component for C10_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(325) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_3, s1: C9_13, s2: C9_2, s3: C9_6)]
pub struct C10_16(pub u64);
impl Component for C10_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(326) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_0, s1: C9_15, s2: C9_4, s3: C9_8)]
pub struct C10_17(pub u64);
impl Component for C10_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(327) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_6, s1: C9_14, s2: C9_12, s3: C9_10)]
pub struct C10_18(pub u64);
impl Component for C10_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(328) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C9_8, s1: C9_19, s2: C9_7, s3: C9_17)]
pub struct C10_19(pub u64);
impl Component for C10_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C10_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(329) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_1, s1: C10_18, s2: C10_5, s3: C10_11)]
pub struct C11_0(pub u64);
impl Component for C11_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(341) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_13, s1: C10_17, s2: C10_16, s3: C10_1)]
pub struct C11_1(pub u64);
impl Component for C11_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(342) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_11, s1: C10_17, s2: C10_13, s3: C10_6)]
pub struct C11_2(pub u64);
impl Component for C11_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(343) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_17, s1: C10_13, s2: C10_2, s3: C10_8)]
pub struct C11_3(pub u64);
impl Component for C11_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(344) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_19, s1: C10_2, s2: C10_8, s3: C10_5)]
pub struct C11_4(pub u64);
impl Component for C11_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(345) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_3, s1: C10_4, s2: C10_1, s3: C10_6)]
pub struct C11_5(pub u64);
impl Component for C11_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(346) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_13, s1: C10_1, s2: C10_18, s3: C10_2)]
pub struct C11_6(pub u64);
impl Component for C11_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(347) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_16, s1: C10_15, s2: C10_19, s3: C10_11)]
pub struct C11_7(pub u64);
impl Component for C11_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(348) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_3, s1: C10_10, s2: C10_1, s3: C10_4)]
pub struct C11_8(pub u64);
impl Component for C11_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(349) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_17, s1: C10_1, s2: C10_14, s3: C10_4)]
pub struct C11_9(pub u64);
impl Component for C11_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(350) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_12, s1: C10_14, s2: C10_0, s3: C10_16)]
pub struct C11_10(pub u64);
impl Component for C11_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(351) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_8, s1: C10_2, s2: C10_19, s3: C10_10)]
pub struct C11_11(pub u64);
impl Component for C11_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(352) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_2, s1: C10_9, s2: C10_1, s3: C10_12)]
pub struct C11_12(pub u64);
impl Component for C11_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(353) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_1, s1: C10_8, s2: C10_10, s3: C10_4)]
pub struct C11_13(pub u64);
impl Component for C11_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(354) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_8, s1: C10_12, s2: C10_3, s3: C10_9)]
pub struct C11_14(pub u64);
impl Component for C11_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(355) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_3, s1: C10_13, s2: C10_7, s3: C10_16)]
pub struct C11_15(pub u64);
impl Component for C11_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(356) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_17, s1: C10_6, s2: C10_10, s3: C10_19)]
pub struct C11_16(pub u64);
impl Component for C11_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(357) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_16, s1: C10_12, s2: C10_15, s3: C10_3)]
pub struct C11_17(pub u64);
impl Component for C11_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(358) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_4, s1: C10_14, s2: C10_16, s3: C10_17)]
pub struct C11_18(pub u64);
impl Component for C11_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(359) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C10_17, s1: C10_0, s2: C10_9, s3: C10_5)]
pub struct C11_19(pub u64);
impl Component for C11_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C11_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(360) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_6, s1: C11_11, s2: C11_12, s3: C11_16)]
pub struct C12_0(pub u64);
impl Component for C12_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(372) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_10, s1: C11_3, s2: C11_13, s3: C11_11)]
pub struct C12_1(pub u64);
impl Component for C12_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(373) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_4, s1: C11_18, s2: C11_2, s3: C11_1)]
pub struct C12_2(pub u64);
impl Component for C12_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(374) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_9, s1: C11_17, s2: C11_10, s3: C11_13)]
pub struct C12_3(pub u64);
impl Component for C12_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(375) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_9, s1: C11_10, s2: C11_11, s3: C11_8)]
pub struct C12_4(pub u64);
impl Component for C12_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(376) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_10, s1: C11_16, s2: C11_18, s3: C11_0)]
pub struct C12_5(pub u64);
impl Component for C12_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(377) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_16, s1: C11_3, s2: C11_4, s3: C11_10)]
pub struct C12_6(pub u64);
impl Component for C12_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(378) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_10, s1: C11_19, s2: C11_2, s3: C11_14)]
pub struct C12_7(pub u64);
impl Component for C12_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(379) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_8, s1: C11_15, s2: C11_14, s3: C11_11)]
pub struct C12_8(pub u64);
impl Component for C12_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(380) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_12, s1: C11_2, s2: C11_1, s3: C11_4)]
pub struct C12_9(pub u64);
impl Component for C12_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(381) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_1, s1: C11_16, s2: C11_15, s3: C11_8)]
pub struct C12_10(pub u64);
impl Component for C12_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(382) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_7, s1: C11_18, s2: C11_10, s3: C11_11)]
pub struct C12_11(pub u64);
impl Component for C12_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(383) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_11, s1: C11_12, s2: C11_9, s3: C11_14)]
pub struct C12_12(pub u64);
impl Component for C12_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(384) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_19, s1: C11_10, s2: C11_17, s3: C11_16)]
pub struct C12_13(pub u64);
impl Component for C12_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(385) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_5, s1: C11_0, s2: C11_4, s3: C11_8)]
pub struct C12_14(pub u64);
impl Component for C12_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(386) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_7, s1: C11_18, s2: C11_4, s3: C11_3)]
pub struct C12_15(pub u64);
impl Component for C12_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(387) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_5, s1: C11_13, s2: C11_1, s3: C11_3)]
pub struct C12_16(pub u64);
impl Component for C12_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(388) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_17, s1: C11_8, s2: C11_3, s3: C11_6)]
pub struct C12_17(pub u64);
impl Component for C12_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(389) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_8, s1: C11_2, s2: C11_16, s3: C11_18)]
pub struct C12_18(pub u64);
impl Component for C12_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(390) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C11_2, s1: C11_6, s2: C11_5, s3: C11_16)]
pub struct C12_19(pub u64);
impl Component for C12_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C12_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(391) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_13, s1: C12_0, s2: C12_11, s3: C12_15)]
pub struct C13_0(pub u64);
impl Component for C13_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(403) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_9, s1: C12_7, s2: C12_6, s3: C12_15)]
pub struct C13_1(pub u64);
impl Component for C13_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(404) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_7, s1: C12_13, s2: C12_14, s3: C12_11)]
pub struct C13_2(pub u64);
impl Component for C13_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(405) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_17, s1: C12_6, s2: C12_15, s3: C12_2)]
pub struct C13_3(pub u64);
impl Component for C13_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(406) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_8, s1: C12_13, s2: C12_6, s3: C12_0)]
pub struct C13_4(pub u64);
impl Component for C13_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(407) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_17, s1: C12_12, s2: C12_16, s3: C12_15)]
pub struct C13_5(pub u64);
impl Component for C13_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(408) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_2, s1: C12_12, s2: C12_16, s3: C12_13)]
pub struct C13_6(pub u64);
impl Component for C13_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(409) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_1, s1: C12_11, s2: C12_14, s3: C12_0)]
pub struct C13_7(pub u64);
impl Component for C13_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(410) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_6, s1: C12_9, s2: C12_0, s3: C12_3)]
pub struct C13_8(pub u64);
impl Component for C13_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(411) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_9, s1: C12_16, s2: C12_10, s3: C12_19)]
pub struct C13_9(pub u64);
impl Component for C13_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(412) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_16, s1: C12_13, s2: C12_17, s3: C12_19)]
pub struct C13_10(pub u64);
impl Component for C13_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(413) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_13, s1: C12_18, s2: C12_9, s3: C12_14)]
pub struct C13_11(pub u64);
impl Component for C13_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(414) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_9, s1: C12_4, s2: C12_16, s3: C12_14)]
pub struct C13_12(pub u64);
impl Component for C13_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(415) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_18, s1: C12_4, s2: C12_17, s3: C12_5)]
pub struct C13_13(pub u64);
impl Component for C13_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(416) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_8, s1: C12_0, s2: C12_13, s3: C12_1)]
pub struct C13_14(pub u64);
impl Component for C13_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(417) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_11, s1: C12_13, s2: C12_12, s3: C12_9)]
pub struct C13_15(pub u64);
impl Component for C13_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(418) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_0, s1: C12_2, s2: C12_18, s3: C12_19)]
pub struct C13_16(pub u64);
impl Component for C13_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(419) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_12, s1: C12_8, s2: C12_14, s3: C12_18)]
pub struct C13_17(pub u64);
impl Component for C13_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(420) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_11, s1: C12_15, s2: C12_10, s3: C12_12)]
pub struct C13_18(pub u64);
impl Component for C13_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(421) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C12_14, s1: C12_3, s2: C12_15, s3: C12_11)]
pub struct C13_19(pub u64);
impl Component for C13_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C13_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(422) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_4, s1: C13_13, s2: C13_19, s3: C13_0)]
pub struct C14_0(pub u64);
impl Component for C14_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(434) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_5, s1: C13_8, s2: C13_11, s3: C13_4)]
pub struct C14_1(pub u64);
impl Component for C14_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(435) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_18, s1: C13_9, s2: C13_13, s3: C13_8)]
pub struct C14_2(pub u64);
impl Component for C14_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(436) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_16, s1: C13_9, s2: C13_13, s3: C13_8)]
pub struct C14_3(pub u64);
impl Component for C14_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(437) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_13, s1: C13_10, s2: C13_15, s3: C13_6)]
pub struct C14_4(pub u64);
impl Component for C14_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(438) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_15, s1: C13_12, s2: C13_13, s3: C13_2)]
pub struct C14_5(pub u64);
impl Component for C14_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(439) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_2, s1: C13_4, s2: C13_6, s3: C13_18)]
pub struct C14_6(pub u64);
impl Component for C14_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(440) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_7, s1: C13_0, s2: C13_3, s3: C13_8)]
pub struct C14_7(pub u64);
impl Component for C14_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(441) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_4, s1: C13_15, s2: C13_3, s3: C13_12)]
pub struct C14_8(pub u64);
impl Component for C14_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(442) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_5, s1: C13_0, s2: C13_2, s3: C13_13)]
pub struct C14_9(pub u64);
impl Component for C14_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(443) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_19, s1: C13_1, s2: C13_17, s3: C13_6)]
pub struct C14_10(pub u64);
impl Component for C14_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(444) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_17, s1: C13_13, s2: C13_11, s3: C13_1)]
pub struct C14_11(pub u64);
impl Component for C14_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(445) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_3, s1: C13_17, s2: C13_13, s3: C13_19)]
pub struct C14_12(pub u64);
impl Component for C14_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(446) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_8, s1: C13_19, s2: C13_5, s3: C13_15)]
pub struct C14_13(pub u64);
impl Component for C14_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(447) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_1, s1: C13_6, s2: C13_2, s3: C13_12)]
pub struct C14_14(pub u64);
impl Component for C14_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(448) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_3, s1: C13_14, s2: C13_9, s3: C13_16)]
pub struct C14_15(pub u64);
impl Component for C14_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(449) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_15, s1: C13_12, s2: C13_3, s3: C13_19)]
pub struct C14_16(pub u64);
impl Component for C14_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(450) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_3, s1: C13_4, s2: C13_12, s3: C13_6)]
pub struct C14_17(pub u64);
impl Component for C14_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(451) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_5, s1: C13_16, s2: C13_8, s3: C13_13)]
pub struct C14_18(pub u64);
impl Component for C14_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(452) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C13_17, s1: C13_9, s2: C13_15, s3: C13_6)]
pub struct C14_19(pub u64);
impl Component for C14_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C14_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(453) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_19, s1: C14_10, s2: C14_15, s3: C14_3)]
pub struct C15_0(pub u64);
impl Component for C15_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(465) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_0, s1: C14_11, s2: C14_8, s3: C14_1)]
pub struct C15_1(pub u64);
impl Component for C15_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(466) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_17, s1: C14_14, s2: C14_9, s3: C14_3)]
pub struct C15_2(pub u64);
impl Component for C15_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(467) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_7, s1: C14_16, s2: C14_8, s3: C14_17)]
pub struct C15_3(pub u64);
impl Component for C15_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(468) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_7, s1: C14_13, s2: C14_4, s3: C14_17)]
pub struct C15_4(pub u64);
impl Component for C15_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(469) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_8, s1: C14_6, s2: C14_13, s3: C14_1)]
pub struct C15_5(pub u64);
impl Component for C15_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(470) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_17, s1: C14_16, s2: C14_4, s3: C14_13)]
pub struct C15_6(pub u64);
impl Component for C15_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(471) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_8, s1: C14_19, s2: C14_15, s3: C14_9)]
pub struct C15_7(pub u64);
impl Component for C15_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(472) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_8, s1: C14_15, s2: C14_6, s3: C14_18)]
pub struct C15_8(pub u64);
impl Component for C15_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(473) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_11, s1: C14_15, s2: C14_7, s3: C14_10)]
pub struct C15_9(pub u64);
impl Component for C15_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(474) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_5, s1: C14_19, s2: C14_14, s3: C14_4)]
pub struct C15_10(pub u64);
impl Component for C15_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(475) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_1, s1: C14_16, s2: C14_10, s3: C14_18)]
pub struct C15_11(pub u64);
impl Component for C15_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(476) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_4, s1: C14_6, s2: C14_10, s3: C14_15)]
pub struct C15_12(pub u64);
impl Component for C15_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(477) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_15, s1: C14_10, s2: C14_3, s3: C14_4)]
pub struct C15_13(pub u64);
impl Component for C15_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(478) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_4, s1: C14_8, s2: C14_7, s3: C14_2)]
pub struct C15_14(pub u64);
impl Component for C15_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(479) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_17, s1: C14_1, s2: C14_5, s3: C14_3)]
pub struct C15_15(pub u64);
impl Component for C15_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(480) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_7, s1: C14_18, s2: C14_6, s3: C14_16)]
pub struct C15_16(pub u64);
impl Component for C15_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(481) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_18, s1: C14_9, s2: C14_13, s3: C14_10)]
pub struct C15_17(pub u64);
impl Component for C15_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(482) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_0, s1: C14_19, s2: C14_9, s3: C14_7)]
pub struct C15_18(pub u64);
impl Component for C15_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(483) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C14_2, s1: C14_7, s2: C14_8, s3: C14_10)]
pub struct C15_19(pub u64);
impl Component for C15_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C15_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(484) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_8, s1: C15_16, s2: C15_12, s3: C15_0)]
pub struct C16_0(pub u64);
impl Component for C16_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(496) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_3, s1: C15_10, s2: C15_11, s3: C15_4)]
pub struct C16_1(pub u64);
impl Component for C16_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(497) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_3, s1: C15_8, s2: C15_4, s3: C15_1)]
pub struct C16_2(pub u64);
impl Component for C16_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(498) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_11, s1: C15_2, s2: C15_18, s3: C15_3)]
pub struct C16_3(pub u64);
impl Component for C16_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(499) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_9, s1: C15_10, s2: C15_7, s3: C15_8)]
pub struct C16_4(pub u64);
impl Component for C16_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(500) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_16, s1: C15_1, s2: C15_11, s3: C15_0)]
pub struct C16_5(pub u64);
impl Component for C16_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(501) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_2, s1: C15_4, s2: C15_12, s3: C15_11)]
pub struct C16_6(pub u64);
impl Component for C16_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(502) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_7, s1: C15_3, s2: C15_10, s3: C15_8)]
pub struct C16_7(pub u64);
impl Component for C16_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(503) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_0, s1: C15_16, s2: C15_10, s3: C15_3)]
pub struct C16_8(pub u64);
impl Component for C16_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(504) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_11, s1: C15_4, s2: C15_8, s3: C15_12)]
pub struct C16_9(pub u64);
impl Component for C16_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(505) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_2, s1: C15_18, s2: C15_16, s3: C15_15)]
pub struct C16_10(pub u64);
impl Component for C16_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(506) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_18, s1: C15_13, s2: C15_17, s3: C15_12)]
pub struct C16_11(pub u64);
impl Component for C16_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(507) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_9, s1: C15_7, s2: C15_19, s3: C15_4)]
pub struct C16_12(pub u64);
impl Component for C16_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(508) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_1, s1: C15_16, s2: C15_3, s3: C15_5)]
pub struct C16_13(pub u64);
impl Component for C16_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(509) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_7, s1: C15_6, s2: C15_13, s3: C15_8)]
pub struct C16_14(pub u64);
impl Component for C16_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(510) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_17, s1: C15_0, s2: C15_8, s3: C15_19)]
pub struct C16_15(pub u64);
impl Component for C16_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(511) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_16, s1: C15_8, s2: C15_15, s3: C15_4)]
pub struct C16_16(pub u64);
impl Component for C16_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(512) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_12, s1: C15_3, s2: C15_11, s3: C15_2)]
pub struct C16_17(pub u64);
impl Component for C16_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(513) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_17, s1: C15_11, s2: C15_19, s3: C15_16)]
pub struct C16_18(pub u64);
impl Component for C16_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(514) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C15_18, s1: C15_0, s2: C15_9, s3: C15_14)]
pub struct C16_19(pub u64);
impl Component for C16_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C16_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(515) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_4, s1: C16_19, s2: C16_2, s3: C16_18)]
pub struct C17_0(pub u64);
impl Component for C17_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(527) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_6, s1: C16_15, s2: C16_10, s3: C16_11)]
pub struct C17_1(pub u64);
impl Component for C17_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(528) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_9, s1: C16_5, s2: C16_4, s3: C16_12)]
pub struct C17_2(pub u64);
impl Component for C17_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(529) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_14, s1: C16_12, s2: C16_3, s3: C16_4)]
pub struct C17_3(pub u64);
impl Component for C17_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(530) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_8, s1: C16_9, s2: C16_0, s3: C16_17)]
pub struct C17_4(pub u64);
impl Component for C17_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(531) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_4, s1: C16_12, s2: C16_17, s3: C16_3)]
pub struct C17_5(pub u64);
impl Component for C17_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(532) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_14, s1: C16_0, s2: C16_13, s3: C16_17)]
pub struct C17_6(pub u64);
impl Component for C17_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(533) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_8, s1: C16_11, s2: C16_13, s3: C16_12)]
pub struct C17_7(pub u64);
impl Component for C17_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(534) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_19, s1: C16_14, s2: C16_1, s3: C16_3)]
pub struct C17_8(pub u64);
impl Component for C17_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(535) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_15, s1: C16_1, s2: C16_0, s3: C16_18)]
pub struct C17_9(pub u64);
impl Component for C17_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(536) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_3, s1: C16_18, s2: C16_4, s3: C16_16)]
pub struct C17_10(pub u64);
impl Component for C17_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(537) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_16, s1: C16_11, s2: C16_17, s3: C16_8)]
pub struct C17_11(pub u64);
impl Component for C17_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(538) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_18, s1: C16_11, s2: C16_15, s3: C16_7)]
pub struct C17_12(pub u64);
impl Component for C17_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(539) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_19, s1: C16_7, s2: C16_3, s3: C16_11)]
pub struct C17_13(pub u64);
impl Component for C17_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(540) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_5, s1: C16_3, s2: C16_1, s3: C16_10)]
pub struct C17_14(pub u64);
impl Component for C17_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(541) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_13, s1: C16_11, s2: C16_8, s3: C16_1)]
pub struct C17_15(pub u64);
impl Component for C17_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(542) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_19, s1: C16_13, s2: C16_18, s3: C16_12)]
pub struct C17_16(pub u64);
impl Component for C17_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(543) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_11, s1: C16_9, s2: C16_10, s3: C16_14)]
pub struct C17_17(pub u64);
impl Component for C17_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(544) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_7, s1: C16_16, s2: C16_4, s3: C16_1)]
pub struct C17_18(pub u64);
impl Component for C17_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(545) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C16_10, s1: C16_3, s2: C16_16, s3: C16_5)]
pub struct C17_19(pub u64);
impl Component for C17_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C17_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(546) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_17, s1: C17_15, s2: C17_10, s3: C17_3)]
pub struct C18_0(pub u64);
impl Component for C18_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(558) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_18, s1: C17_0, s2: C17_15, s3: C17_6)]
pub struct C18_1(pub u64);
impl Component for C18_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(559) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_12, s1: C17_5, s2: C17_19, s3: C17_7)]
pub struct C18_2(pub u64);
impl Component for C18_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(560) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_3, s1: C17_7, s2: C17_10, s3: C17_17)]
pub struct C18_3(pub u64);
impl Component for C18_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(561) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_7, s1: C17_14, s2: C17_15, s3: C17_11)]
pub struct C18_4(pub u64);
impl Component for C18_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(562) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_15, s1: C17_6, s2: C17_13, s3: C17_14)]
pub struct C18_5(pub u64);
impl Component for C18_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(563) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_12, s1: C17_17, s2: C17_3, s3: C17_15)]
pub struct C18_6(pub u64);
impl Component for C18_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(564) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_8, s1: C17_4, s2: C17_18, s3: C17_0)]
pub struct C18_7(pub u64);
impl Component for C18_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(565) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_12, s1: C17_13, s2: C17_3, s3: C17_0)]
pub struct C18_8(pub u64);
impl Component for C18_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(566) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_2, s1: C17_5, s2: C17_14, s3: C17_12)]
pub struct C18_9(pub u64);
impl Component for C18_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(567) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_16, s1: C17_9, s2: C17_4, s3: C17_17)]
pub struct C18_10(pub u64);
impl Component for C18_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(568) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_16, s1: C17_3, s2: C17_8, s3: C17_0)]
pub struct C18_11(pub u64);
impl Component for C18_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(569) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_14, s1: C17_12, s2: C17_7, s3: C17_18)]
pub struct C18_12(pub u64);
impl Component for C18_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(570) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_0, s1: C17_17, s2: C17_7, s3: C17_13)]
pub struct C18_13(pub u64);
impl Component for C18_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(571) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_5, s1: C17_19, s2: C17_10, s3: C17_7)]
pub struct C18_14(pub u64);
impl Component for C18_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(572) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_2, s1: C17_17, s2: C17_18, s3: C17_5)]
pub struct C18_15(pub u64);
impl Component for C18_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(573) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_5, s1: C17_12, s2: C17_0, s3: C17_16)]
pub struct C18_16(pub u64);
impl Component for C18_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(574) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_6, s1: C17_13, s2: C17_7, s3: C17_1)]
pub struct C18_17(pub u64);
impl Component for C18_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(575) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_16, s1: C17_6, s2: C17_19, s3: C17_2)]
pub struct C18_18(pub u64);
impl Component for C18_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(576) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C17_7, s1: C17_12, s2: C17_14, s3: C17_3)]
pub struct C18_19(pub u64);
impl Component for C18_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C18_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(577) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_18, s1: C18_1, s2: C18_12, s3: C18_2)]
pub struct C19_0(pub u64);
impl Component for C19_0 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_0(0), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(589) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_17, s1: C18_3, s2: C18_15, s3: C18_1)]
pub struct C19_1(pub u64);
impl Component for C19_1 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_1(1), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(590) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_16, s1: C18_7, s2: C18_0, s3: C18_17)]
pub struct C19_2(pub u64);
impl Component for C19_2 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_2(2), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(591) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_9, s1: C18_14, s2: C18_8, s3: C18_13)]
pub struct C19_3(pub u64);
impl Component for C19_3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_3(3), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(592) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_5, s1: C18_4, s2: C18_17, s3: C18_10)]
pub struct C19_4(pub u64);
impl Component for C19_4 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_4(4), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(593) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_17, s1: C18_14, s2: C18_16, s3: C18_13)]
pub struct C19_5(pub u64);
impl Component for C19_5 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_5(5), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(594) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_17, s1: C18_5, s2: C18_12, s3: C18_19)]
pub struct C19_6(pub u64);
impl Component for C19_6 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_6(6), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(595) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_6, s1: C18_15, s2: C18_8, s3: C18_11)]
pub struct C19_7(pub u64);
impl Component for C19_7 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_7(7), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(596) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_4, s1: C18_8, s2: C18_18, s3: C18_5)]
pub struct C19_8(pub u64);
impl Component for C19_8 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_8(8), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(597) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_19, s1: C18_2, s2: C18_11, s3: C18_10)]
pub struct C19_9(pub u64);
impl Component for C19_9 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_9(9), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(598) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_4, s1: C18_8, s2: C18_18, s3: C18_17)]
pub struct C19_10(pub u64);
impl Component for C19_10 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_10(10), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(599) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_11, s1: C18_12, s2: C18_8, s3: C18_14)]
pub struct C19_11(pub u64);
impl Component for C19_11 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_11(11), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(600) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_0, s1: C18_4, s2: C18_18, s3: C18_8)]
pub struct C19_12(pub u64);
impl Component for C19_12 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_12(12), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(601) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_7, s1: C18_6, s2: C18_2, s3: C18_18)]
pub struct C19_13(pub u64);
impl Component for C19_13 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_13(13), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(602) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_17, s1: C18_13, s2: C18_7, s3: C18_4)]
pub struct C19_14(pub u64);
impl Component for C19_14 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_14(14), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(603) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_17, s1: C18_14, s2: C18_12, s3: C18_6)]
pub struct C19_15(pub u64);
impl Component for C19_15 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_15(15), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(604) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_2, s1: C18_19, s2: C18_4, s3: C18_1)]
pub struct C19_16(pub u64);
impl Component for C19_16 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_16(16), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(605) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_0, s1: C18_12, s2: C18_18, s3: C18_13)]
pub struct C19_17(pub u64);
impl Component for C19_17 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_17(17), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(606) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_4, s1: C18_18, s2: C18_19, s3: C18_2)]
pub struct C19_18(pub u64);
impl Component for C19_18 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_18(18), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(607) }
}
#[derive(ComponentSubscription)]
#[subscribe(s0: C18_7, s1: C18_12, s2: C18_4, s3: C18_9)]
pub struct C19_19(pub u64);
impl Component for C19_19 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (C19_19(19), 0) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 { (input.s0.rotate_left(1) ^ input.s1.rotate_left(2) ^ input.s2.rotate_left(3) ^ input.s3.rotate_left(4)).wrapping_add(608) }
}
#[derive(ComponentSubscription)]
#[subscribe(r0: C19_0, r1: C19_1, r2: C19_2, r3: C19_3, r4: C19_4, r5: C19_5, r6: C19_6, r7: C19_7, r8: C19_8, r9: C19_9, r10: C19_10, r11: C19_11, r12: C19_12, r13: C19_13, r14: C19_14, r15: C19_15, r16: C19_16, r17: C19_17, r18: C19_18, r19: C19_19)]
pub struct App;
impl Component for App {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) { (App, ()) }
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) { OUT.store(*input.r0 ^ *input.r1 ^ *input.r2 ^ *input.r3 ^ *input.r4 ^ *input.r5 ^ *input.r6 ^ *input.r7 ^ *input.r8 ^ *input.r9 ^ *input.r10 ^ *input.r11 ^ *input.r12 ^ *input.r13 ^ *input.r14 ^ *input.r15 ^ *input.r16 ^ *input.r17 ^ *input.r18 ^ *input.r19, Ordering::Relaxed) }
}
}
mod hand_components {
pub struct C0_0(pub u64);
impl C0_0 {
    pub fn new() -> Self { C0_0(0) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1); self.0 }
}
pub struct C0_1(pub u64);
impl C0_1 {
    pub fn new() -> Self { C0_1(1) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(2); self.0 }
}
pub struct C0_2(pub u64);
impl C0_2 {
    pub fn new() -> Self { C0_2(2) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(3); self.0 }
}
pub struct C0_3(pub u64);
impl C0_3 {
    pub fn new() -> Self { C0_3(3) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(4); self.0 }
}
pub struct C0_4(pub u64);
impl C0_4 {
    pub fn new() -> Self { C0_4(4) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(5); self.0 }
}
pub struct C0_5(pub u64);
impl C0_5 {
    pub fn new() -> Self { C0_5(5) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(6); self.0 }
}
pub struct C0_6(pub u64);
impl C0_6 {
    pub fn new() -> Self { C0_6(6) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(7); self.0 }
}
pub struct C0_7(pub u64);
impl C0_7 {
    pub fn new() -> Self { C0_7(7) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(8); self.0 }
}
pub struct C0_8(pub u64);
impl C0_8 {
    pub fn new() -> Self { C0_8(8) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(9); self.0 }
}
pub struct C0_9(pub u64);
impl C0_9 {
    pub fn new() -> Self { C0_9(9) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(10); self.0 }
}
pub struct C0_10(pub u64);
impl C0_10 {
    pub fn new() -> Self { C0_10(10) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(11); self.0 }
}
pub struct C0_11(pub u64);
impl C0_11 {
    pub fn new() -> Self { C0_11(11) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(12); self.0 }
}
pub struct C0_12(pub u64);
impl C0_12 {
    pub fn new() -> Self { C0_12(12) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(13); self.0 }
}
pub struct C0_13(pub u64);
impl C0_13 {
    pub fn new() -> Self { C0_13(13) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(14); self.0 }
}
pub struct C0_14(pub u64);
impl C0_14 {
    pub fn new() -> Self { C0_14(14) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(15); self.0 }
}
pub struct C0_15(pub u64);
impl C0_15 {
    pub fn new() -> Self { C0_15(15) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(16); self.0 }
}
pub struct C0_16(pub u64);
impl C0_16 {
    pub fn new() -> Self { C0_16(16) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(17); self.0 }
}
pub struct C0_17(pub u64);
impl C0_17 {
    pub fn new() -> Self { C0_17(17) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(18); self.0 }
}
pub struct C0_18(pub u64);
impl C0_18 {
    pub fn new() -> Self { C0_18(18) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(19); self.0 }
}
pub struct C0_19(pub u64);
impl C0_19 {
    pub fn new() -> Self { C0_19(19) }
    #[inline]
    pub fn update(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(20); self.0 }
}
pub struct C1_0(pub u64);
impl C1_0 {
    pub fn new() -> Self { C1_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(31) }
}
pub struct C1_1(pub u64);
impl C1_1 {
    pub fn new() -> Self { C1_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(32) }
}
pub struct C1_2(pub u64);
impl C1_2 {
    pub fn new() -> Self { C1_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(33) }
}
pub struct C1_3(pub u64);
impl C1_3 {
    pub fn new() -> Self { C1_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(34) }
}
pub struct C1_4(pub u64);
impl C1_4 {
    pub fn new() -> Self { C1_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(35) }
}
pub struct C1_5(pub u64);
impl C1_5 {
    pub fn new() -> Self { C1_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(36) }
}
pub struct C1_6(pub u64);
impl C1_6 {
    pub fn new() -> Self { C1_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(37) }
}
pub struct C1_7(pub u64);
impl C1_7 {
    pub fn new() -> Self { C1_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(38) }
}
pub struct C1_8(pub u64);
impl C1_8 {
    pub fn new() -> Self { C1_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(39) }
}
pub struct C1_9(pub u64);
impl C1_9 {
    pub fn new() -> Self { C1_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(40) }
}
pub struct C1_10(pub u64);
impl C1_10 {
    pub fn new() -> Self { C1_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(41) }
}
pub struct C1_11(pub u64);
impl C1_11 {
    pub fn new() -> Self { C1_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(42) }
}
pub struct C1_12(pub u64);
impl C1_12 {
    pub fn new() -> Self { C1_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(43) }
}
pub struct C1_13(pub u64);
impl C1_13 {
    pub fn new() -> Self { C1_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(44) }
}
pub struct C1_14(pub u64);
impl C1_14 {
    pub fn new() -> Self { C1_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(45) }
}
pub struct C1_15(pub u64);
impl C1_15 {
    pub fn new() -> Self { C1_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(46) }
}
pub struct C1_16(pub u64);
impl C1_16 {
    pub fn new() -> Self { C1_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(47) }
}
pub struct C1_17(pub u64);
impl C1_17 {
    pub fn new() -> Self { C1_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(48) }
}
pub struct C1_18(pub u64);
impl C1_18 {
    pub fn new() -> Self { C1_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(49) }
}
pub struct C1_19(pub u64);
impl C1_19 {
    pub fn new() -> Self { C1_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(50) }
}
pub struct C2_0(pub u64);
impl C2_0 {
    pub fn new() -> Self { C2_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(62) }
}
pub struct C2_1(pub u64);
impl C2_1 {
    pub fn new() -> Self { C2_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(63) }
}
pub struct C2_2(pub u64);
impl C2_2 {
    pub fn new() -> Self { C2_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(64) }
}
pub struct C2_3(pub u64);
impl C2_3 {
    pub fn new() -> Self { C2_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(65) }
}
pub struct C2_4(pub u64);
impl C2_4 {
    pub fn new() -> Self { C2_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(66) }
}
pub struct C2_5(pub u64);
impl C2_5 {
    pub fn new() -> Self { C2_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(67) }
}
pub struct C2_6(pub u64);
impl C2_6 {
    pub fn new() -> Self { C2_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(68) }
}
pub struct C2_7(pub u64);
impl C2_7 {
    pub fn new() -> Self { C2_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(69) }
}
pub struct C2_8(pub u64);
impl C2_8 {
    pub fn new() -> Self { C2_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(70) }
}
pub struct C2_9(pub u64);
impl C2_9 {
    pub fn new() -> Self { C2_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(71) }
}
pub struct C2_10(pub u64);
impl C2_10 {
    pub fn new() -> Self { C2_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(72) }
}
pub struct C2_11(pub u64);
impl C2_11 {
    pub fn new() -> Self { C2_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(73) }
}
pub struct C2_12(pub u64);
impl C2_12 {
    pub fn new() -> Self { C2_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(74) }
}
pub struct C2_13(pub u64);
impl C2_13 {
    pub fn new() -> Self { C2_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(75) }
}
pub struct C2_14(pub u64);
impl C2_14 {
    pub fn new() -> Self { C2_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(76) }
}
pub struct C2_15(pub u64);
impl C2_15 {
    pub fn new() -> Self { C2_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(77) }
}
pub struct C2_16(pub u64);
impl C2_16 {
    pub fn new() -> Self { C2_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(78) }
}
pub struct C2_17(pub u64);
impl C2_17 {
    pub fn new() -> Self { C2_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(79) }
}
pub struct C2_18(pub u64);
impl C2_18 {
    pub fn new() -> Self { C2_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(80) }
}
pub struct C2_19(pub u64);
impl C2_19 {
    pub fn new() -> Self { C2_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(81) }
}
pub struct C3_0(pub u64);
impl C3_0 {
    pub fn new() -> Self { C3_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(93) }
}
pub struct C3_1(pub u64);
impl C3_1 {
    pub fn new() -> Self { C3_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(94) }
}
pub struct C3_2(pub u64);
impl C3_2 {
    pub fn new() -> Self { C3_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(95) }
}
pub struct C3_3(pub u64);
impl C3_3 {
    pub fn new() -> Self { C3_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(96) }
}
pub struct C3_4(pub u64);
impl C3_4 {
    pub fn new() -> Self { C3_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(97) }
}
pub struct C3_5(pub u64);
impl C3_5 {
    pub fn new() -> Self { C3_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(98) }
}
pub struct C3_6(pub u64);
impl C3_6 {
    pub fn new() -> Self { C3_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(99) }
}
pub struct C3_7(pub u64);
impl C3_7 {
    pub fn new() -> Self { C3_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(100) }
}
pub struct C3_8(pub u64);
impl C3_8 {
    pub fn new() -> Self { C3_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(101) }
}
pub struct C3_9(pub u64);
impl C3_9 {
    pub fn new() -> Self { C3_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(102) }
}
pub struct C3_10(pub u64);
impl C3_10 {
    pub fn new() -> Self { C3_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(103) }
}
pub struct C3_11(pub u64);
impl C3_11 {
    pub fn new() -> Self { C3_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(104) }
}
pub struct C3_12(pub u64);
impl C3_12 {
    pub fn new() -> Self { C3_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(105) }
}
pub struct C3_13(pub u64);
impl C3_13 {
    pub fn new() -> Self { C3_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(106) }
}
pub struct C3_14(pub u64);
impl C3_14 {
    pub fn new() -> Self { C3_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(107) }
}
pub struct C3_15(pub u64);
impl C3_15 {
    pub fn new() -> Self { C3_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(108) }
}
pub struct C3_16(pub u64);
impl C3_16 {
    pub fn new() -> Self { C3_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(109) }
}
pub struct C3_17(pub u64);
impl C3_17 {
    pub fn new() -> Self { C3_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(110) }
}
pub struct C3_18(pub u64);
impl C3_18 {
    pub fn new() -> Self { C3_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(111) }
}
pub struct C3_19(pub u64);
impl C3_19 {
    pub fn new() -> Self { C3_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(112) }
}
pub struct C4_0(pub u64);
impl C4_0 {
    pub fn new() -> Self { C4_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(124) }
}
pub struct C4_1(pub u64);
impl C4_1 {
    pub fn new() -> Self { C4_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(125) }
}
pub struct C4_2(pub u64);
impl C4_2 {
    pub fn new() -> Self { C4_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(126) }
}
pub struct C4_3(pub u64);
impl C4_3 {
    pub fn new() -> Self { C4_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(127) }
}
pub struct C4_4(pub u64);
impl C4_4 {
    pub fn new() -> Self { C4_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(128) }
}
pub struct C4_5(pub u64);
impl C4_5 {
    pub fn new() -> Self { C4_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(129) }
}
pub struct C4_6(pub u64);
impl C4_6 {
    pub fn new() -> Self { C4_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(130) }
}
pub struct C4_7(pub u64);
impl C4_7 {
    pub fn new() -> Self { C4_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(131) }
}
pub struct C4_8(pub u64);
impl C4_8 {
    pub fn new() -> Self { C4_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(132) }
}
pub struct C4_9(pub u64);
impl C4_9 {
    pub fn new() -> Self { C4_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(133) }
}
pub struct C4_10(pub u64);
impl C4_10 {
    pub fn new() -> Self { C4_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(134) }
}
pub struct C4_11(pub u64);
impl C4_11 {
    pub fn new() -> Self { C4_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(135) }
}
pub struct C4_12(pub u64);
impl C4_12 {
    pub fn new() -> Self { C4_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(136) }
}
pub struct C4_13(pub u64);
impl C4_13 {
    pub fn new() -> Self { C4_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(137) }
}
pub struct C4_14(pub u64);
impl C4_14 {
    pub fn new() -> Self { C4_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(138) }
}
pub struct C4_15(pub u64);
impl C4_15 {
    pub fn new() -> Self { C4_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(139) }
}
pub struct C4_16(pub u64);
impl C4_16 {
    pub fn new() -> Self { C4_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(140) }
}
pub struct C4_17(pub u64);
impl C4_17 {
    pub fn new() -> Self { C4_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(141) }
}
pub struct C4_18(pub u64);
impl C4_18 {
    pub fn new() -> Self { C4_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(142) }
}
pub struct C4_19(pub u64);
impl C4_19 {
    pub fn new() -> Self { C4_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(143) }
}
pub struct C5_0(pub u64);
impl C5_0 {
    pub fn new() -> Self { C5_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(155) }
}
pub struct C5_1(pub u64);
impl C5_1 {
    pub fn new() -> Self { C5_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(156) }
}
pub struct C5_2(pub u64);
impl C5_2 {
    pub fn new() -> Self { C5_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(157) }
}
pub struct C5_3(pub u64);
impl C5_3 {
    pub fn new() -> Self { C5_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(158) }
}
pub struct C5_4(pub u64);
impl C5_4 {
    pub fn new() -> Self { C5_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(159) }
}
pub struct C5_5(pub u64);
impl C5_5 {
    pub fn new() -> Self { C5_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(160) }
}
pub struct C5_6(pub u64);
impl C5_6 {
    pub fn new() -> Self { C5_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(161) }
}
pub struct C5_7(pub u64);
impl C5_7 {
    pub fn new() -> Self { C5_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(162) }
}
pub struct C5_8(pub u64);
impl C5_8 {
    pub fn new() -> Self { C5_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(163) }
}
pub struct C5_9(pub u64);
impl C5_9 {
    pub fn new() -> Self { C5_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(164) }
}
pub struct C5_10(pub u64);
impl C5_10 {
    pub fn new() -> Self { C5_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(165) }
}
pub struct C5_11(pub u64);
impl C5_11 {
    pub fn new() -> Self { C5_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(166) }
}
pub struct C5_12(pub u64);
impl C5_12 {
    pub fn new() -> Self { C5_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(167) }
}
pub struct C5_13(pub u64);
impl C5_13 {
    pub fn new() -> Self { C5_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(168) }
}
pub struct C5_14(pub u64);
impl C5_14 {
    pub fn new() -> Self { C5_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(169) }
}
pub struct C5_15(pub u64);
impl C5_15 {
    pub fn new() -> Self { C5_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(170) }
}
pub struct C5_16(pub u64);
impl C5_16 {
    pub fn new() -> Self { C5_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(171) }
}
pub struct C5_17(pub u64);
impl C5_17 {
    pub fn new() -> Self { C5_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(172) }
}
pub struct C5_18(pub u64);
impl C5_18 {
    pub fn new() -> Self { C5_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(173) }
}
pub struct C5_19(pub u64);
impl C5_19 {
    pub fn new() -> Self { C5_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(174) }
}
pub struct C6_0(pub u64);
impl C6_0 {
    pub fn new() -> Self { C6_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(186) }
}
pub struct C6_1(pub u64);
impl C6_1 {
    pub fn new() -> Self { C6_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(187) }
}
pub struct C6_2(pub u64);
impl C6_2 {
    pub fn new() -> Self { C6_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(188) }
}
pub struct C6_3(pub u64);
impl C6_3 {
    pub fn new() -> Self { C6_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(189) }
}
pub struct C6_4(pub u64);
impl C6_4 {
    pub fn new() -> Self { C6_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(190) }
}
pub struct C6_5(pub u64);
impl C6_5 {
    pub fn new() -> Self { C6_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(191) }
}
pub struct C6_6(pub u64);
impl C6_6 {
    pub fn new() -> Self { C6_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(192) }
}
pub struct C6_7(pub u64);
impl C6_7 {
    pub fn new() -> Self { C6_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(193) }
}
pub struct C6_8(pub u64);
impl C6_8 {
    pub fn new() -> Self { C6_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(194) }
}
pub struct C6_9(pub u64);
impl C6_9 {
    pub fn new() -> Self { C6_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(195) }
}
pub struct C6_10(pub u64);
impl C6_10 {
    pub fn new() -> Self { C6_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(196) }
}
pub struct C6_11(pub u64);
impl C6_11 {
    pub fn new() -> Self { C6_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(197) }
}
pub struct C6_12(pub u64);
impl C6_12 {
    pub fn new() -> Self { C6_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(198) }
}
pub struct C6_13(pub u64);
impl C6_13 {
    pub fn new() -> Self { C6_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(199) }
}
pub struct C6_14(pub u64);
impl C6_14 {
    pub fn new() -> Self { C6_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(200) }
}
pub struct C6_15(pub u64);
impl C6_15 {
    pub fn new() -> Self { C6_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(201) }
}
pub struct C6_16(pub u64);
impl C6_16 {
    pub fn new() -> Self { C6_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(202) }
}
pub struct C6_17(pub u64);
impl C6_17 {
    pub fn new() -> Self { C6_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(203) }
}
pub struct C6_18(pub u64);
impl C6_18 {
    pub fn new() -> Self { C6_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(204) }
}
pub struct C6_19(pub u64);
impl C6_19 {
    pub fn new() -> Self { C6_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(205) }
}
pub struct C7_0(pub u64);
impl C7_0 {
    pub fn new() -> Self { C7_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(217) }
}
pub struct C7_1(pub u64);
impl C7_1 {
    pub fn new() -> Self { C7_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(218) }
}
pub struct C7_2(pub u64);
impl C7_2 {
    pub fn new() -> Self { C7_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(219) }
}
pub struct C7_3(pub u64);
impl C7_3 {
    pub fn new() -> Self { C7_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(220) }
}
pub struct C7_4(pub u64);
impl C7_4 {
    pub fn new() -> Self { C7_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(221) }
}
pub struct C7_5(pub u64);
impl C7_5 {
    pub fn new() -> Self { C7_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(222) }
}
pub struct C7_6(pub u64);
impl C7_6 {
    pub fn new() -> Self { C7_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(223) }
}
pub struct C7_7(pub u64);
impl C7_7 {
    pub fn new() -> Self { C7_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(224) }
}
pub struct C7_8(pub u64);
impl C7_8 {
    pub fn new() -> Self { C7_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(225) }
}
pub struct C7_9(pub u64);
impl C7_9 {
    pub fn new() -> Self { C7_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(226) }
}
pub struct C7_10(pub u64);
impl C7_10 {
    pub fn new() -> Self { C7_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(227) }
}
pub struct C7_11(pub u64);
impl C7_11 {
    pub fn new() -> Self { C7_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(228) }
}
pub struct C7_12(pub u64);
impl C7_12 {
    pub fn new() -> Self { C7_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(229) }
}
pub struct C7_13(pub u64);
impl C7_13 {
    pub fn new() -> Self { C7_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(230) }
}
pub struct C7_14(pub u64);
impl C7_14 {
    pub fn new() -> Self { C7_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(231) }
}
pub struct C7_15(pub u64);
impl C7_15 {
    pub fn new() -> Self { C7_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(232) }
}
pub struct C7_16(pub u64);
impl C7_16 {
    pub fn new() -> Self { C7_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(233) }
}
pub struct C7_17(pub u64);
impl C7_17 {
    pub fn new() -> Self { C7_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(234) }
}
pub struct C7_18(pub u64);
impl C7_18 {
    pub fn new() -> Self { C7_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(235) }
}
pub struct C7_19(pub u64);
impl C7_19 {
    pub fn new() -> Self { C7_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(236) }
}
pub struct C8_0(pub u64);
impl C8_0 {
    pub fn new() -> Self { C8_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(248) }
}
pub struct C8_1(pub u64);
impl C8_1 {
    pub fn new() -> Self { C8_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(249) }
}
pub struct C8_2(pub u64);
impl C8_2 {
    pub fn new() -> Self { C8_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(250) }
}
pub struct C8_3(pub u64);
impl C8_3 {
    pub fn new() -> Self { C8_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(251) }
}
pub struct C8_4(pub u64);
impl C8_4 {
    pub fn new() -> Self { C8_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(252) }
}
pub struct C8_5(pub u64);
impl C8_5 {
    pub fn new() -> Self { C8_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(253) }
}
pub struct C8_6(pub u64);
impl C8_6 {
    pub fn new() -> Self { C8_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(254) }
}
pub struct C8_7(pub u64);
impl C8_7 {
    pub fn new() -> Self { C8_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(255) }
}
pub struct C8_8(pub u64);
impl C8_8 {
    pub fn new() -> Self { C8_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(256) }
}
pub struct C8_9(pub u64);
impl C8_9 {
    pub fn new() -> Self { C8_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(257) }
}
pub struct C8_10(pub u64);
impl C8_10 {
    pub fn new() -> Self { C8_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(258) }
}
pub struct C8_11(pub u64);
impl C8_11 {
    pub fn new() -> Self { C8_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(259) }
}
pub struct C8_12(pub u64);
impl C8_12 {
    pub fn new() -> Self { C8_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(260) }
}
pub struct C8_13(pub u64);
impl C8_13 {
    pub fn new() -> Self { C8_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(261) }
}
pub struct C8_14(pub u64);
impl C8_14 {
    pub fn new() -> Self { C8_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(262) }
}
pub struct C8_15(pub u64);
impl C8_15 {
    pub fn new() -> Self { C8_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(263) }
}
pub struct C8_16(pub u64);
impl C8_16 {
    pub fn new() -> Self { C8_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(264) }
}
pub struct C8_17(pub u64);
impl C8_17 {
    pub fn new() -> Self { C8_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(265) }
}
pub struct C8_18(pub u64);
impl C8_18 {
    pub fn new() -> Self { C8_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(266) }
}
pub struct C8_19(pub u64);
impl C8_19 {
    pub fn new() -> Self { C8_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(267) }
}
pub struct C9_0(pub u64);
impl C9_0 {
    pub fn new() -> Self { C9_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(279) }
}
pub struct C9_1(pub u64);
impl C9_1 {
    pub fn new() -> Self { C9_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(280) }
}
pub struct C9_2(pub u64);
impl C9_2 {
    pub fn new() -> Self { C9_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(281) }
}
pub struct C9_3(pub u64);
impl C9_3 {
    pub fn new() -> Self { C9_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(282) }
}
pub struct C9_4(pub u64);
impl C9_4 {
    pub fn new() -> Self { C9_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(283) }
}
pub struct C9_5(pub u64);
impl C9_5 {
    pub fn new() -> Self { C9_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(284) }
}
pub struct C9_6(pub u64);
impl C9_6 {
    pub fn new() -> Self { C9_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(285) }
}
pub struct C9_7(pub u64);
impl C9_7 {
    pub fn new() -> Self { C9_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(286) }
}
pub struct C9_8(pub u64);
impl C9_8 {
    pub fn new() -> Self { C9_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(287) }
}
pub struct C9_9(pub u64);
impl C9_9 {
    pub fn new() -> Self { C9_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(288) }
}
pub struct C9_10(pub u64);
impl C9_10 {
    pub fn new() -> Self { C9_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(289) }
}
pub struct C9_11(pub u64);
impl C9_11 {
    pub fn new() -> Self { C9_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(290) }
}
pub struct C9_12(pub u64);
impl C9_12 {
    pub fn new() -> Self { C9_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(291) }
}
pub struct C9_13(pub u64);
impl C9_13 {
    pub fn new() -> Self { C9_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(292) }
}
pub struct C9_14(pub u64);
impl C9_14 {
    pub fn new() -> Self { C9_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(293) }
}
pub struct C9_15(pub u64);
impl C9_15 {
    pub fn new() -> Self { C9_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(294) }
}
pub struct C9_16(pub u64);
impl C9_16 {
    pub fn new() -> Self { C9_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(295) }
}
pub struct C9_17(pub u64);
impl C9_17 {
    pub fn new() -> Self { C9_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(296) }
}
pub struct C9_18(pub u64);
impl C9_18 {
    pub fn new() -> Self { C9_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(297) }
}
pub struct C9_19(pub u64);
impl C9_19 {
    pub fn new() -> Self { C9_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(298) }
}
pub struct C10_0(pub u64);
impl C10_0 {
    pub fn new() -> Self { C10_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(310) }
}
pub struct C10_1(pub u64);
impl C10_1 {
    pub fn new() -> Self { C10_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(311) }
}
pub struct C10_2(pub u64);
impl C10_2 {
    pub fn new() -> Self { C10_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(312) }
}
pub struct C10_3(pub u64);
impl C10_3 {
    pub fn new() -> Self { C10_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(313) }
}
pub struct C10_4(pub u64);
impl C10_4 {
    pub fn new() -> Self { C10_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(314) }
}
pub struct C10_5(pub u64);
impl C10_5 {
    pub fn new() -> Self { C10_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(315) }
}
pub struct C10_6(pub u64);
impl C10_6 {
    pub fn new() -> Self { C10_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(316) }
}
pub struct C10_7(pub u64);
impl C10_7 {
    pub fn new() -> Self { C10_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(317) }
}
pub struct C10_8(pub u64);
impl C10_8 {
    pub fn new() -> Self { C10_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(318) }
}
pub struct C10_9(pub u64);
impl C10_9 {
    pub fn new() -> Self { C10_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(319) }
}
pub struct C10_10(pub u64);
impl C10_10 {
    pub fn new() -> Self { C10_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(320) }
}
pub struct C10_11(pub u64);
impl C10_11 {
    pub fn new() -> Self { C10_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(321) }
}
pub struct C10_12(pub u64);
impl C10_12 {
    pub fn new() -> Self { C10_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(322) }
}
pub struct C10_13(pub u64);
impl C10_13 {
    pub fn new() -> Self { C10_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(323) }
}
pub struct C10_14(pub u64);
impl C10_14 {
    pub fn new() -> Self { C10_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(324) }
}
pub struct C10_15(pub u64);
impl C10_15 {
    pub fn new() -> Self { C10_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(325) }
}
pub struct C10_16(pub u64);
impl C10_16 {
    pub fn new() -> Self { C10_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(326) }
}
pub struct C10_17(pub u64);
impl C10_17 {
    pub fn new() -> Self { C10_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(327) }
}
pub struct C10_18(pub u64);
impl C10_18 {
    pub fn new() -> Self { C10_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(328) }
}
pub struct C10_19(pub u64);
impl C10_19 {
    pub fn new() -> Self { C10_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(329) }
}
pub struct C11_0(pub u64);
impl C11_0 {
    pub fn new() -> Self { C11_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(341) }
}
pub struct C11_1(pub u64);
impl C11_1 {
    pub fn new() -> Self { C11_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(342) }
}
pub struct C11_2(pub u64);
impl C11_2 {
    pub fn new() -> Self { C11_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(343) }
}
pub struct C11_3(pub u64);
impl C11_3 {
    pub fn new() -> Self { C11_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(344) }
}
pub struct C11_4(pub u64);
impl C11_4 {
    pub fn new() -> Self { C11_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(345) }
}
pub struct C11_5(pub u64);
impl C11_5 {
    pub fn new() -> Self { C11_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(346) }
}
pub struct C11_6(pub u64);
impl C11_6 {
    pub fn new() -> Self { C11_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(347) }
}
pub struct C11_7(pub u64);
impl C11_7 {
    pub fn new() -> Self { C11_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(348) }
}
pub struct C11_8(pub u64);
impl C11_8 {
    pub fn new() -> Self { C11_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(349) }
}
pub struct C11_9(pub u64);
impl C11_9 {
    pub fn new() -> Self { C11_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(350) }
}
pub struct C11_10(pub u64);
impl C11_10 {
    pub fn new() -> Self { C11_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(351) }
}
pub struct C11_11(pub u64);
impl C11_11 {
    pub fn new() -> Self { C11_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(352) }
}
pub struct C11_12(pub u64);
impl C11_12 {
    pub fn new() -> Self { C11_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(353) }
}
pub struct C11_13(pub u64);
impl C11_13 {
    pub fn new() -> Self { C11_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(354) }
}
pub struct C11_14(pub u64);
impl C11_14 {
    pub fn new() -> Self { C11_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(355) }
}
pub struct C11_15(pub u64);
impl C11_15 {
    pub fn new() -> Self { C11_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(356) }
}
pub struct C11_16(pub u64);
impl C11_16 {
    pub fn new() -> Self { C11_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(357) }
}
pub struct C11_17(pub u64);
impl C11_17 {
    pub fn new() -> Self { C11_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(358) }
}
pub struct C11_18(pub u64);
impl C11_18 {
    pub fn new() -> Self { C11_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(359) }
}
pub struct C11_19(pub u64);
impl C11_19 {
    pub fn new() -> Self { C11_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(360) }
}
pub struct C12_0(pub u64);
impl C12_0 {
    pub fn new() -> Self { C12_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(372) }
}
pub struct C12_1(pub u64);
impl C12_1 {
    pub fn new() -> Self { C12_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(373) }
}
pub struct C12_2(pub u64);
impl C12_2 {
    pub fn new() -> Self { C12_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(374) }
}
pub struct C12_3(pub u64);
impl C12_3 {
    pub fn new() -> Self { C12_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(375) }
}
pub struct C12_4(pub u64);
impl C12_4 {
    pub fn new() -> Self { C12_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(376) }
}
pub struct C12_5(pub u64);
impl C12_5 {
    pub fn new() -> Self { C12_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(377) }
}
pub struct C12_6(pub u64);
impl C12_6 {
    pub fn new() -> Self { C12_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(378) }
}
pub struct C12_7(pub u64);
impl C12_7 {
    pub fn new() -> Self { C12_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(379) }
}
pub struct C12_8(pub u64);
impl C12_8 {
    pub fn new() -> Self { C12_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(380) }
}
pub struct C12_9(pub u64);
impl C12_9 {
    pub fn new() -> Self { C12_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(381) }
}
pub struct C12_10(pub u64);
impl C12_10 {
    pub fn new() -> Self { C12_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(382) }
}
pub struct C12_11(pub u64);
impl C12_11 {
    pub fn new() -> Self { C12_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(383) }
}
pub struct C12_12(pub u64);
impl C12_12 {
    pub fn new() -> Self { C12_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(384) }
}
pub struct C12_13(pub u64);
impl C12_13 {
    pub fn new() -> Self { C12_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(385) }
}
pub struct C12_14(pub u64);
impl C12_14 {
    pub fn new() -> Self { C12_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(386) }
}
pub struct C12_15(pub u64);
impl C12_15 {
    pub fn new() -> Self { C12_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(387) }
}
pub struct C12_16(pub u64);
impl C12_16 {
    pub fn new() -> Self { C12_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(388) }
}
pub struct C12_17(pub u64);
impl C12_17 {
    pub fn new() -> Self { C12_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(389) }
}
pub struct C12_18(pub u64);
impl C12_18 {
    pub fn new() -> Self { C12_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(390) }
}
pub struct C12_19(pub u64);
impl C12_19 {
    pub fn new() -> Self { C12_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(391) }
}
pub struct C13_0(pub u64);
impl C13_0 {
    pub fn new() -> Self { C13_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(403) }
}
pub struct C13_1(pub u64);
impl C13_1 {
    pub fn new() -> Self { C13_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(404) }
}
pub struct C13_2(pub u64);
impl C13_2 {
    pub fn new() -> Self { C13_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(405) }
}
pub struct C13_3(pub u64);
impl C13_3 {
    pub fn new() -> Self { C13_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(406) }
}
pub struct C13_4(pub u64);
impl C13_4 {
    pub fn new() -> Self { C13_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(407) }
}
pub struct C13_5(pub u64);
impl C13_5 {
    pub fn new() -> Self { C13_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(408) }
}
pub struct C13_6(pub u64);
impl C13_6 {
    pub fn new() -> Self { C13_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(409) }
}
pub struct C13_7(pub u64);
impl C13_7 {
    pub fn new() -> Self { C13_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(410) }
}
pub struct C13_8(pub u64);
impl C13_8 {
    pub fn new() -> Self { C13_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(411) }
}
pub struct C13_9(pub u64);
impl C13_9 {
    pub fn new() -> Self { C13_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(412) }
}
pub struct C13_10(pub u64);
impl C13_10 {
    pub fn new() -> Self { C13_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(413) }
}
pub struct C13_11(pub u64);
impl C13_11 {
    pub fn new() -> Self { C13_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(414) }
}
pub struct C13_12(pub u64);
impl C13_12 {
    pub fn new() -> Self { C13_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(415) }
}
pub struct C13_13(pub u64);
impl C13_13 {
    pub fn new() -> Self { C13_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(416) }
}
pub struct C13_14(pub u64);
impl C13_14 {
    pub fn new() -> Self { C13_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(417) }
}
pub struct C13_15(pub u64);
impl C13_15 {
    pub fn new() -> Self { C13_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(418) }
}
pub struct C13_16(pub u64);
impl C13_16 {
    pub fn new() -> Self { C13_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(419) }
}
pub struct C13_17(pub u64);
impl C13_17 {
    pub fn new() -> Self { C13_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(420) }
}
pub struct C13_18(pub u64);
impl C13_18 {
    pub fn new() -> Self { C13_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(421) }
}
pub struct C13_19(pub u64);
impl C13_19 {
    pub fn new() -> Self { C13_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(422) }
}
pub struct C14_0(pub u64);
impl C14_0 {
    pub fn new() -> Self { C14_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(434) }
}
pub struct C14_1(pub u64);
impl C14_1 {
    pub fn new() -> Self { C14_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(435) }
}
pub struct C14_2(pub u64);
impl C14_2 {
    pub fn new() -> Self { C14_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(436) }
}
pub struct C14_3(pub u64);
impl C14_3 {
    pub fn new() -> Self { C14_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(437) }
}
pub struct C14_4(pub u64);
impl C14_4 {
    pub fn new() -> Self { C14_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(438) }
}
pub struct C14_5(pub u64);
impl C14_5 {
    pub fn new() -> Self { C14_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(439) }
}
pub struct C14_6(pub u64);
impl C14_6 {
    pub fn new() -> Self { C14_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(440) }
}
pub struct C14_7(pub u64);
impl C14_7 {
    pub fn new() -> Self { C14_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(441) }
}
pub struct C14_8(pub u64);
impl C14_8 {
    pub fn new() -> Self { C14_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(442) }
}
pub struct C14_9(pub u64);
impl C14_9 {
    pub fn new() -> Self { C14_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(443) }
}
pub struct C14_10(pub u64);
impl C14_10 {
    pub fn new() -> Self { C14_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(444) }
}
pub struct C14_11(pub u64);
impl C14_11 {
    pub fn new() -> Self { C14_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(445) }
}
pub struct C14_12(pub u64);
impl C14_12 {
    pub fn new() -> Self { C14_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(446) }
}
pub struct C14_13(pub u64);
impl C14_13 {
    pub fn new() -> Self { C14_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(447) }
}
pub struct C14_14(pub u64);
impl C14_14 {
    pub fn new() -> Self { C14_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(448) }
}
pub struct C14_15(pub u64);
impl C14_15 {
    pub fn new() -> Self { C14_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(449) }
}
pub struct C14_16(pub u64);
impl C14_16 {
    pub fn new() -> Self { C14_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(450) }
}
pub struct C14_17(pub u64);
impl C14_17 {
    pub fn new() -> Self { C14_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(451) }
}
pub struct C14_18(pub u64);
impl C14_18 {
    pub fn new() -> Self { C14_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(452) }
}
pub struct C14_19(pub u64);
impl C14_19 {
    pub fn new() -> Self { C14_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(453) }
}
pub struct C15_0(pub u64);
impl C15_0 {
    pub fn new() -> Self { C15_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(465) }
}
pub struct C15_1(pub u64);
impl C15_1 {
    pub fn new() -> Self { C15_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(466) }
}
pub struct C15_2(pub u64);
impl C15_2 {
    pub fn new() -> Self { C15_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(467) }
}
pub struct C15_3(pub u64);
impl C15_3 {
    pub fn new() -> Self { C15_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(468) }
}
pub struct C15_4(pub u64);
impl C15_4 {
    pub fn new() -> Self { C15_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(469) }
}
pub struct C15_5(pub u64);
impl C15_5 {
    pub fn new() -> Self { C15_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(470) }
}
pub struct C15_6(pub u64);
impl C15_6 {
    pub fn new() -> Self { C15_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(471) }
}
pub struct C15_7(pub u64);
impl C15_7 {
    pub fn new() -> Self { C15_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(472) }
}
pub struct C15_8(pub u64);
impl C15_8 {
    pub fn new() -> Self { C15_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(473) }
}
pub struct C15_9(pub u64);
impl C15_9 {
    pub fn new() -> Self { C15_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(474) }
}
pub struct C15_10(pub u64);
impl C15_10 {
    pub fn new() -> Self { C15_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(475) }
}
pub struct C15_11(pub u64);
impl C15_11 {
    pub fn new() -> Self { C15_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(476) }
}
pub struct C15_12(pub u64);
impl C15_12 {
    pub fn new() -> Self { C15_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(477) }
}
pub struct C15_13(pub u64);
impl C15_13 {
    pub fn new() -> Self { C15_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(478) }
}
pub struct C15_14(pub u64);
impl C15_14 {
    pub fn new() -> Self { C15_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(479) }
}
pub struct C15_15(pub u64);
impl C15_15 {
    pub fn new() -> Self { C15_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(480) }
}
pub struct C15_16(pub u64);
impl C15_16 {
    pub fn new() -> Self { C15_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(481) }
}
pub struct C15_17(pub u64);
impl C15_17 {
    pub fn new() -> Self { C15_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(482) }
}
pub struct C15_18(pub u64);
impl C15_18 {
    pub fn new() -> Self { C15_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(483) }
}
pub struct C15_19(pub u64);
impl C15_19 {
    pub fn new() -> Self { C15_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(484) }
}
pub struct C16_0(pub u64);
impl C16_0 {
    pub fn new() -> Self { C16_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(496) }
}
pub struct C16_1(pub u64);
impl C16_1 {
    pub fn new() -> Self { C16_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(497) }
}
pub struct C16_2(pub u64);
impl C16_2 {
    pub fn new() -> Self { C16_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(498) }
}
pub struct C16_3(pub u64);
impl C16_3 {
    pub fn new() -> Self { C16_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(499) }
}
pub struct C16_4(pub u64);
impl C16_4 {
    pub fn new() -> Self { C16_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(500) }
}
pub struct C16_5(pub u64);
impl C16_5 {
    pub fn new() -> Self { C16_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(501) }
}
pub struct C16_6(pub u64);
impl C16_6 {
    pub fn new() -> Self { C16_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(502) }
}
pub struct C16_7(pub u64);
impl C16_7 {
    pub fn new() -> Self { C16_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(503) }
}
pub struct C16_8(pub u64);
impl C16_8 {
    pub fn new() -> Self { C16_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(504) }
}
pub struct C16_9(pub u64);
impl C16_9 {
    pub fn new() -> Self { C16_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(505) }
}
pub struct C16_10(pub u64);
impl C16_10 {
    pub fn new() -> Self { C16_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(506) }
}
pub struct C16_11(pub u64);
impl C16_11 {
    pub fn new() -> Self { C16_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(507) }
}
pub struct C16_12(pub u64);
impl C16_12 {
    pub fn new() -> Self { C16_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(508) }
}
pub struct C16_13(pub u64);
impl C16_13 {
    pub fn new() -> Self { C16_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(509) }
}
pub struct C16_14(pub u64);
impl C16_14 {
    pub fn new() -> Self { C16_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(510) }
}
pub struct C16_15(pub u64);
impl C16_15 {
    pub fn new() -> Self { C16_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(511) }
}
pub struct C16_16(pub u64);
impl C16_16 {
    pub fn new() -> Self { C16_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(512) }
}
pub struct C16_17(pub u64);
impl C16_17 {
    pub fn new() -> Self { C16_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(513) }
}
pub struct C16_18(pub u64);
impl C16_18 {
    pub fn new() -> Self { C16_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(514) }
}
pub struct C16_19(pub u64);
impl C16_19 {
    pub fn new() -> Self { C16_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(515) }
}
pub struct C17_0(pub u64);
impl C17_0 {
    pub fn new() -> Self { C17_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(527) }
}
pub struct C17_1(pub u64);
impl C17_1 {
    pub fn new() -> Self { C17_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(528) }
}
pub struct C17_2(pub u64);
impl C17_2 {
    pub fn new() -> Self { C17_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(529) }
}
pub struct C17_3(pub u64);
impl C17_3 {
    pub fn new() -> Self { C17_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(530) }
}
pub struct C17_4(pub u64);
impl C17_4 {
    pub fn new() -> Self { C17_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(531) }
}
pub struct C17_5(pub u64);
impl C17_5 {
    pub fn new() -> Self { C17_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(532) }
}
pub struct C17_6(pub u64);
impl C17_6 {
    pub fn new() -> Self { C17_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(533) }
}
pub struct C17_7(pub u64);
impl C17_7 {
    pub fn new() -> Self { C17_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(534) }
}
pub struct C17_8(pub u64);
impl C17_8 {
    pub fn new() -> Self { C17_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(535) }
}
pub struct C17_9(pub u64);
impl C17_9 {
    pub fn new() -> Self { C17_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(536) }
}
pub struct C17_10(pub u64);
impl C17_10 {
    pub fn new() -> Self { C17_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(537) }
}
pub struct C17_11(pub u64);
impl C17_11 {
    pub fn new() -> Self { C17_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(538) }
}
pub struct C17_12(pub u64);
impl C17_12 {
    pub fn new() -> Self { C17_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(539) }
}
pub struct C17_13(pub u64);
impl C17_13 {
    pub fn new() -> Self { C17_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(540) }
}
pub struct C17_14(pub u64);
impl C17_14 {
    pub fn new() -> Self { C17_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(541) }
}
pub struct C17_15(pub u64);
impl C17_15 {
    pub fn new() -> Self { C17_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(542) }
}
pub struct C17_16(pub u64);
impl C17_16 {
    pub fn new() -> Self { C17_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(543) }
}
pub struct C17_17(pub u64);
impl C17_17 {
    pub fn new() -> Self { C17_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(544) }
}
pub struct C17_18(pub u64);
impl C17_18 {
    pub fn new() -> Self { C17_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(545) }
}
pub struct C17_19(pub u64);
impl C17_19 {
    pub fn new() -> Self { C17_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(546) }
}
pub struct C18_0(pub u64);
impl C18_0 {
    pub fn new() -> Self { C18_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(558) }
}
pub struct C18_1(pub u64);
impl C18_1 {
    pub fn new() -> Self { C18_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(559) }
}
pub struct C18_2(pub u64);
impl C18_2 {
    pub fn new() -> Self { C18_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(560) }
}
pub struct C18_3(pub u64);
impl C18_3 {
    pub fn new() -> Self { C18_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(561) }
}
pub struct C18_4(pub u64);
impl C18_4 {
    pub fn new() -> Self { C18_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(562) }
}
pub struct C18_5(pub u64);
impl C18_5 {
    pub fn new() -> Self { C18_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(563) }
}
pub struct C18_6(pub u64);
impl C18_6 {
    pub fn new() -> Self { C18_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(564) }
}
pub struct C18_7(pub u64);
impl C18_7 {
    pub fn new() -> Self { C18_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(565) }
}
pub struct C18_8(pub u64);
impl C18_8 {
    pub fn new() -> Self { C18_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(566) }
}
pub struct C18_9(pub u64);
impl C18_9 {
    pub fn new() -> Self { C18_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(567) }
}
pub struct C18_10(pub u64);
impl C18_10 {
    pub fn new() -> Self { C18_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(568) }
}
pub struct C18_11(pub u64);
impl C18_11 {
    pub fn new() -> Self { C18_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(569) }
}
pub struct C18_12(pub u64);
impl C18_12 {
    pub fn new() -> Self { C18_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(570) }
}
pub struct C18_13(pub u64);
impl C18_13 {
    pub fn new() -> Self { C18_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(571) }
}
pub struct C18_14(pub u64);
impl C18_14 {
    pub fn new() -> Self { C18_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(572) }
}
pub struct C18_15(pub u64);
impl C18_15 {
    pub fn new() -> Self { C18_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(573) }
}
pub struct C18_16(pub u64);
impl C18_16 {
    pub fn new() -> Self { C18_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(574) }
}
pub struct C18_17(pub u64);
impl C18_17 {
    pub fn new() -> Self { C18_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(575) }
}
pub struct C18_18(pub u64);
impl C18_18 {
    pub fn new() -> Self { C18_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(576) }
}
pub struct C18_19(pub u64);
impl C18_19 {
    pub fn new() -> Self { C18_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(577) }
}
pub struct C19_0(pub u64);
impl C19_0 {
    pub fn new() -> Self { C19_0(0) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(589) }
}
pub struct C19_1(pub u64);
impl C19_1 {
    pub fn new() -> Self { C19_1(1) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(590) }
}
pub struct C19_2(pub u64);
impl C19_2 {
    pub fn new() -> Self { C19_2(2) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(591) }
}
pub struct C19_3(pub u64);
impl C19_3 {
    pub fn new() -> Self { C19_3(3) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(592) }
}
pub struct C19_4(pub u64);
impl C19_4 {
    pub fn new() -> Self { C19_4(4) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(593) }
}
pub struct C19_5(pub u64);
impl C19_5 {
    pub fn new() -> Self { C19_5(5) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(594) }
}
pub struct C19_6(pub u64);
impl C19_6 {
    pub fn new() -> Self { C19_6(6) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(595) }
}
pub struct C19_7(pub u64);
impl C19_7 {
    pub fn new() -> Self { C19_7(7) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(596) }
}
pub struct C19_8(pub u64);
impl C19_8 {
    pub fn new() -> Self { C19_8(8) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(597) }
}
pub struct C19_9(pub u64);
impl C19_9 {
    pub fn new() -> Self { C19_9(9) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(598) }
}
pub struct C19_10(pub u64);
impl C19_10 {
    pub fn new() -> Self { C19_10(10) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(599) }
}
pub struct C19_11(pub u64);
impl C19_11 {
    pub fn new() -> Self { C19_11(11) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(600) }
}
pub struct C19_12(pub u64);
impl C19_12 {
    pub fn new() -> Self { C19_12(12) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(601) }
}
pub struct C19_13(pub u64);
impl C19_13 {
    pub fn new() -> Self { C19_13(13) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(602) }
}
pub struct C19_14(pub u64);
impl C19_14 {
    pub fn new() -> Self { C19_14(14) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(603) }
}
pub struct C19_15(pub u64);
impl C19_15 {
    pub fn new() -> Self { C19_15(15) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(604) }
}
pub struct C19_16(pub u64);
impl C19_16 {
    pub fn new() -> Self { C19_16(16) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(605) }
}
pub struct C19_17(pub u64);
impl C19_17 {
    pub fn new() -> Self { C19_17(17) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(606) }
}
pub struct C19_18(pub u64);
impl C19_18 {
    pub fn new() -> Self { C19_18(18) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(607) }
}
pub struct C19_19(pub u64);
impl C19_19 {
    pub fn new() -> Self { C19_19(19) }
    #[inline]
    pub fn update(&mut self, d0: &u64, d1: &u64, d2: &u64, d3: &u64) -> u64 { (d0.rotate_left(1) ^ d1.rotate_left(2) ^ d2.rotate_left(3) ^ d3.rotate_left(4)).wrapping_add(608) }
}
}
use hand_components::*;
pub struct Hand {
    c_C0_12: C0_12,
    o_C0_12: u64,
    c_C0_18: C0_18,
    o_C0_18: u64,
    c_C0_1: C0_1,
    o_C0_1: u64,
    c_C0_15: C0_15,
    o_C0_15: u64,
    c_C1_14: C1_14,
    o_C1_14: u64,
    c_C0_9: C0_9,
    o_C0_9: u64,
    c_C0_0: C0_0,
    o_C0_0: u64,
    c_C0_13: C0_13,
    o_C0_13: u64,
    c_C0_3: C0_3,
    o_C0_3: u64,
    c_C1_10: C1_10,
    o_C1_10: u64,
    c_C0_7: C0_7,
    o_C0_7: u64,
    c_C0_5: C0_5,
    o_C0_5: u64,
    c_C1_15: C1_15,
    o_C1_15: u64,
    c_C0_14: C0_14,
    o_C0_14: u64,
    c_C0_16: C0_16,
    o_C0_16: u64,
    c_C1_17: C1_17,
    o_C1_17: u64,
    c_C2_18: C2_18,
    o_C2_18: u64,
    c_C0_4: C0_4,
    o_C0_4: u64,
    c_C0_2: C0_2,
    o_C0_2: u64,
    c_C0_8: C0_8,
    o_C0_8: u64,
    c_C1_0: C1_0,
    o_C1_0: u64,
    c_C1_7: C1_7,
    o_C1_7: u64,
    c_C0_10: C0_10,
    o_C0_10: u64,
    c_C0_17: C0_17,
    o_C0_17: u64,
    c_C1_5: C1_5,
    o_C1_5: u64,
    c_C2_10: C2_10,
    o_C2_10: u64,
    c_C0_6: C0_6,
    o_C0_6: u64,
    c_C1_2: C1_2,
    o_C1_2: u64,
    c_C0_11: C0_11,
    o_C0_11: u64,
    c_C1_18: C1_18,
    o_C1_18: u64,
    c_C2_13: C2_13,
    o_C2_13: u64,
    c_C0_19: C0_19,
    o_C0_19: u64,
    c_C1_16: C1_16,
    o_C1_16: u64,
    c_C2_1: C2_1,
    o_C2_1: u64,
    c_C3_7: C3_7,
    o_C3_7: u64,
    c_C1_13: C1_13,
    o_C1_13: u64,
    c_C1_11: C1_11,
    o_C1_11: u64,
    c_C2_9: C2_9,
    o_C2_9: u64,
    c_C1_19: C1_19,
    o_C1_19: u64,
    c_C1_12: C1_12,
    o_C1_12: u64,
    c_C2_0: C2_0,
    o_C2_0: u64,
    c_C1_4: C1_4,
    o_C1_4: u64,
    c_C2_5: C2_5,
    o_C2_5: u64,
    c_C1_6: C1_6,
    o_C1_6: u64,
    c_C1_1: C1_1,
    o_C1_1: u64,
    c_C2_6: C2_6,
    o_C2_6: u64,
    c_C3_18: C3_18,
    o_C3_18: u64,
    c_C1_3: C1_3,
    o_C1_3: u64,
    c_C1_9: C1_9,
    o_C1_9: u64,
    c_C2_19: C2_19,
    o_C2_19: u64,
    c_C1_8: C1_8,
    o_C1_8: u64,
    c_C2_16: C2_16,
    o_C2_16: u64,
    c_C2_12: C2_12,
    o_C2_12: u64,
    c_C3_13: C3_13,
    o_C3_13: u64,
    c_C2_17: C2_17,
    o_C2_17: u64,
    c_C2_7: C2_7,
    o_C2_7: u64,
    c_C2_14: C2_14,
    o_C2_14: u64,
    c_C3_5: C3_5,
    o_C3_5: u64,
    c_C4_16: C4_16,
    o_C4_16: u64,
    c_C2_4: C2_4,
    o_C2_4: u64,
    c_C3_19: C3_19,
    o_C3_19: u64,
    c_C2_3: C2_3,
    o_C2_3: u64,
    c_C3_16: C3_16,
    o_C3_16: u64,
    c_C3_8: C3_8,
    o_C3_8: u64,
    c_C3_11: C3_11,
    o_C3_11: u64,
    c_C4_5: C4_5,
    o_C4_5: u64,
    c_C3_3: C3_3,
    o_C3_3: u64,
    c_C3_12: C3_12,
    o_C3_12: u64,
    c_C4_18: C4_18,
    o_C4_18: u64,
    c_C2_15: C2_15,
    o_C2_15: u64,
    c_C3_17: C3_17,
    o_C3_17: u64,
    c_C3_6: C3_6,
    o_C3_6: u64,
    c_C2_8: C2_8,
    o_C2_8: u64,
    c_C3_10: C3_10,
    o_C3_10: u64,
    c_C4_4: C4_4,
    o_C4_4: u64,
    c_C5_18: C5_18,
    o_C5_18: u64,
    c_C3_2: C3_2,
    o_C3_2: u64,
    c_C4_10: C4_10,
    o_C4_10: u64,
    c_C2_2: C2_2,
    o_C2_2: u64,
    c_C3_9: C3_9,
    o_C3_9: u64,
    c_C3_0: C3_0,
    o_C3_0: u64,
    c_C4_14: C4_14,
    o_C4_14: u64,
    c_C3_4: C3_4,
    o_C3_4: u64,
    c_C4_3: C4_3,
    o_C4_3: u64,
    c_C3_1: C3_1,
    o_C3_1: u64,
    c_C4_8: C4_8,
    o_C4_8: u64,
    c_C5_4: C5_4,
    o_C5_4: u64,
    c_C4_9: C4_9,
    o_C4_9: u64,
    c_C5_19: C5_19,
    o_C5_19: u64,
    c_C4_2: C4_2,
    o_C4_2: u64,
    c_C5_17: C5_17,
    o_C5_17: u64,
    c_C6_10: C6_10,
    o_C6_10: u64,
    c_C4_11: C4_11,
    o_C4_11: u64,
    c_C4_15: C4_15,
    o_C4_15: u64,
    c_C5_16: C5_16,
    o_C5_16: u64,
    c_C4_12: C4_12,
    o_C4_12: u64,
    c_C5_14: C5_14,
    o_C5_14: u64,
    c_C4_6: C4_6,
    o_C4_6: u64,
    c_C3_15: C3_15,
    o_C3_15: u64,
    c_C4_7: C4_7,
    o_C4_7: u64,
    c_C5_7: C5_7,
    o_C5_7: u64,
    c_C6_14: C6_14,
    o_C6_14: u64,
    c_C4_1: C4_1,
    o_C4_1: u64,
    c_C4_0: C4_0,
    o_C4_0: u64,
    c_C2_11: C2_11,
    o_C2_11: u64,
    c_C3_14: C3_14,
    o_C3_14: u64,
    c_C4_17: C4_17,
    o_C4_17: u64,
    c_C5_1: C5_1,
    o_C5_1: u64,
    c_C4_19: C4_19,
    o_C4_19: u64,
    c_C5_8: C5_8,
    o_C5_8: u64,
    c_C5_2: C5_2,
    o_C5_2: u64,
    c_C6_4: C6_4,
    o_C6_4: u64,
    c_C5_10: C5_10,
    o_C5_10: u64,
    c_C5_15: C5_15,
    o_C5_15: u64,
    c_C6_15: C6_15,
    o_C6_15: u64,
    c_C7_9: C7_9,
    o_C7_9: u64,
    c_C5_13: C5_13,
    o_C5_13: u64,
    c_C6_5: C6_5,
    o_C6_5: u64,
    c_C6_17: C6_17,
    o_C6_17: u64,
    c_C5_12: C5_12,
    o_C5_12: u64,
    c_C6_11: C6_11,
    o_C6_11: u64,
    c_C7_17: C7_17,
    o_C7_17: u64,
    c_C5_5: C5_5,
    o_C5_5: u64,
    c_C6_12: C6_12,
    o_C6_12: u64,
    c_C5_9: C5_9,
    o_C5_9: u64,
    c_C5_6: C5_6,
    o_C5_6: u64,
    c_C6_0: C6_0,
    o_C6_0: u64,
    c_C6_6: C6_6,
    o_C6_6: u64,
    c_C7_12: C7_12,
    o_C7_12: u64,
    c_C6_3: C6_3,
    o_C6_3: u64,
    c_C5_0: C5_0,
    o_C5_0: u64,
    c_C6_13: C6_13,
    o_C6_13: u64,
    c_C7_10: C7_10,
    o_C7_10: u64,
    c_C8_5: C8_5,
    o_C8_5: u64,
    c_C5_11: C5_11,
    o_C5_11: u64,
    c_C6_9: C6_9,
    o_C6_9: u64,
    c_C7_19: C7_19,
    o_C7_19: u64,
    c_C6_8: C6_8,
    o_C6_8: u64,
    c_C6_19: C6_19,
    o_C6_19: u64,
    c_C7_16: C7_16,
    o_C7_16: u64,
    c_C6_16: C6_16,
    o_C6_16: u64,
    c_C7_13: C7_13,
    o_C7_13: u64,
    c_C8_9: C8_9,
    o_C8_9: u64,
    c_C7_4: C7_4,
    o_C7_4: u64,
    c_C8_17: C8_17,
    o_C8_17: u64,
    c_C7_8: C7_8,
    o_C7_8: u64,
    c_C6_2: C6_2,
    o_C6_2: u64,
    c_C7_6: C7_6,
    o_C7_6: u64,
    c_C6_7: C6_7,
    o_C6_7: u64,
    c_C7_14: C7_14,
    o_C7_14: u64,
    c_C8_8: C8_8,
    o_C8_8: u64,
    c_C9_7: C9_7,
    o_C9_7: u64,
    c_C7_0: C7_0,
    o_C7_0: u64,
    c_C8_7: C8_7,
    o_C8_7: u64,
    c_C7_18: C7_18,
    o_C7_18: u64,
    c_C7_15: C7_15,
    o_C7_15: u64,
    c_C6_18: C6_18,
    o_C6_18: u64,
    c_C6_1: C6_1,
    o_C6_1: u64,
    c_C7_3: C7_3,
    o_C7_3: u64,
    c_C8_6: C8_6,
    o_C8_6: u64,
    c_C8_2: C8_2,
    o_C8_2: u64,
    c_C9_12: C9_12,
    o_C9_12: u64,
    c_C7_5: C7_5,
    o_C7_5: u64,
    c_C8_15: C8_15,
    o_C8_15: u64,
    c_C7_11: C7_11,
    o_C7_11: u64,
    c_C8_3: C8_3,
    o_C8_3: u64,
    c_C7_2: C7_2,
    o_C7_2: u64,
    c_C8_1: C8_1,
    o_C8_1: u64,
    c_C9_19: C9_19,
    o_C9_19: u64,
    c_C8_4: C8_4,
    o_C8_4: u64,
    c_C8_18: C8_18,
    o_C8_18: u64,
    c_C7_7: C7_7,
    o_C7_7: u64,
    c_C8_14: C8_14,
    o_C8_14: u64,
    c_C9_15: C9_15,
    o_C9_15: u64,
    c_C10_3: C10_3,
    o_C10_3: u64,
    c_C8_11: C8_11,
    o_C8_11: u64,
    c_C8_16: C8_16,
    o_C8_16: u64,
    c_C9_6: C9_6,
    o_C9_6: u64,
    c_C9_5: C9_5,
    o_C9_5: u64,
    c_C8_10: C8_10,
    o_C8_10: u64,
    c_C9_4: C9_4,
    o_C9_4: u64,
    c_C10_10: C10_10,
    o_C10_10: u64,
    c_C8_12: C8_12,
    o_C8_12: u64,
    c_C9_16: C9_16,
    o_C9_16: u64,
    c_C7_1: C7_1,
    o_C7_1: u64,
    c_C8_19: C8_19,
    o_C8_19: u64,
    c_C9_14: C9_14,
    o_C9_14: u64,
    c_C8_0: C8_0,
    o_C8_0: u64,
    c_C9_10: C9_10,
    o_C9_10: u64,
    c_C10_1: C10_1,
    o_C10_1: u64,
    c_C10_4: C10_4,
    o_C10_4: u64,
    c_C11_8: C11_8,
    o_C11_8: u64,
    c_C9_2: C9_2,
    o_C9_2: u64,
    c_C9_9: C9_9,
    o_C9_9: u64,
    c_C10_13: C10_13,
    o_C10_13: u64,
    c_C9_18: C9_18,
    o_C9_18: u64,
    c_C9_3: C9_3,
    o_C9_3: u64,
    c_C10_7: C10_7,
    o_C10_7: u64,
    c_C9_13: C9_13,
    o_C9_13: u64,
    c_C10_16: C10_16,
    o_C10_16: u64,
    c_C11_15: C11_15,
    o_C11_15: u64,
    c_C9_1: C9_1,
    o_C9_1: u64,
    c_C8_13: C8_13,
    o_C8_13: u64,
    c_C9_0: C9_0,
    o_C9_0: u64,
    c_C9_17: C9_17,
    o_C9_17: u64,
    c_C10_8: C10_8,
    o_C10_8: u64,
    c_C9_8: C9_8,
    o_C9_8: u64,
    c_C10_12: C10_12,
    o_C10_12: u64,
    c_C10_9: C10_9,
    o_C10_9: u64,
    c_C11_14: C11_14,
    o_C11_14: u64,
    c_C10_2: C10_2,
    o_C10_2: u64,
    c_C10_19: C10_19,
    o_C10_19: u64,
    c_C11_11: C11_11,
    o_C11_11: u64,
    c_C12_8: C12_8,
    o_C12_8: u64,
    c_C10_17: C10_17,
    o_C10_17: u64,
    c_C10_0: C10_0,
    o_C10_0: u64,
    c_C10_5: C10_5,
    o_C10_5: u64,
    c_C11_19: C11_19,
    o_C11_19: u64,
    c_C10_14: C10_14,
    o_C10_14: u64,
    c_C11_10: C11_10,
    o_C11_10: u64,
    c_C10_15: C10_15,
    o_C10_15: u64,
    c_C11_17: C11_17,
    o_C11_17: u64,
    c_C10_6: C10_6,
    o_C10_6: u64,
    c_C11_16: C11_16,
    o_C11_16: u64,
    c_C12_13: C12_13,
    o_C12_13: u64,
    c_C11_3: C11_3,
    o_C11_3: u64,
    c_C11_4: C11_4,
    o_C11_4: u64,
    c_C12_6: C12_6,
    o_C12_6: u64,
    c_C10_18: C10_18,
    o_C10_18: u64,
    c_C11_6: C11_6,
    o_C11_6: u64,
    c_C11_12: C11_12,
    o_C11_12: u64,
    c_C12_0: C12_0,
    o_C12_0: u64,
    c_C13_4: C13_4,
    o_C13_4: u64,
    c_C10_11: C10_11,
    o_C10_11: u64,
    c_C11_2: C11_2,
    o_C11_2: u64,
    c_C11_18: C11_18,
    o_C11_18: u64,
    c_C12_18: C12_18,
    o_C12_18: u64,
    c_C11_9: C11_9,
    o_C11_9: u64,
    c_C12_4: C12_4,
    o_C12_4: u64,
    c_C12_17: C12_17,
    o_C12_17: u64,
    c_C11_0: C11_0,
    o_C11_0: u64,
    c_C12_5: C12_5,
    o_C12_5: u64,
    c_C13_13: C13_13,
    o_C13_13: u64,
    c_C11_5: C11_5,
    o_C11_5: u64,
    c_C12_14: C12_14,
    o_C12_14: u64,
    c_C11_13: C11_13,
    o_C11_13: u64,
    c_C12_3: C12_3,
    o_C12_3: u64,
    c_C11_7: C11_7,
    o_C11_7: u64,
    c_C12_15: C12_15,
    o_C12_15: u64,
    c_C12_11: C12_11,
    o_C12_11: u64,
    c_C13_19: C13_19,
    o_C13_19: u64,
    c_C13_0: C13_0,
    o_C13_0: u64,
    c_C14_0: C14_0,
    o_C14_0: u64,
    c_C12_12: C12_12,
    o_C12_12: u64,
    c_C13_17: C13_17,
    o_C13_17: u64,
    c_C11_1: C11_1,
    o_C11_1: u64,
    c_C12_9: C12_9,
    o_C12_9: u64,
    c_C12_16: C12_16,
    o_C12_16: u64,
    c_C12_10: C12_10,
    o_C12_10: u64,
    c_C12_19: C12_19,
    o_C12_19: u64,
    c_C13_9: C13_9,
    o_C13_9: u64,
    c_C13_15: C13_15,
    o_C13_15: u64,
    c_C12_2: C12_2,
    o_C12_2: u64,
    c_C13_6: C13_6,
    o_C13_6: u64,
    c_C14_19: C14_19,
    o_C14_19: u64,
    c_C13_5: C13_5,
    o_C13_5: u64,
    c_C12_7: C12_7,
    o_C12_7: u64,
    c_C13_2: C13_2,
    o_C13_2: u64,
    c_C14_9: C14_9,
    o_C14_9: u64,
    c_C12_1: C12_1,
    o_C12_1: u64,
    c_C13_7: C13_7,
    o_C13_7: u64,
    c_C13_3: C13_3,
    o_C13_3: u64,
    c_C13_8: C13_8,
    o_C13_8: u64,
    c_C14_7: C14_7,
    o_C14_7: u64,
    c_C15_18: C15_18,
    o_C15_18: u64,
    c_C13_1: C13_1,
    o_C13_1: u64,
    c_C14_10: C14_10,
    o_C14_10: u64,
    c_C13_14: C13_14,
    o_C13_14: u64,
    c_C13_16: C13_16,
    o_C13_16: u64,
    c_C14_15: C14_15,
    o_C14_15: u64,
    c_C14_3: C14_3,
    o_C14_3: u64,
    c_C15_0: C15_0,
    o_C15_0: u64,
    c_C13_11: C13_11,
    o_C13_11: u64,
    c_C14_11: C14_11,
    o_C14_11: u64,
    c_C15_9: C15_9,
    o_C15_9: u64,
    c_C13_10: C13_10,
    o_C13_10: u64,
    c_C14_4: C14_4,
    o_C14_4: u64,
    c_C13_12: C13_12,
    o_C13_12: u64,
    c_C14_8: C14_8,
    o_C14_8: u64,
    c_C13_18: C13_18,
    o_C13_18: u64,
    c_C14_2: C14_2,
    o_C14_2: u64,
    c_C15_14: C15_14,
    o_C15_14: u64,
    c_C16_19: C16_19,
    o_C16_19: u64,
    c_C14_1: C14_1,
    o_C14_1: u64,
    c_C15_1: C15_1,
    o_C15_1: u64,
    c_C14_18: C14_18,
    o_C14_18: u64,
    c_C14_6: C14_6,
    o_C14_6: u64,
    c_C14_16: C14_16,
    o_C14_16: u64,
    c_C15_16: C15_16,
    o_C15_16: u64,
    c_C14_17: C14_17,
    o_C14_17: u64,
    c_C15_3: C15_3,
    o_C15_3: u64,
    c_C14_13: C14_13,
    o_C14_13: u64,
    c_C15_5: C15_5,
    o_C15_5: u64,
    c_C16_13: C16_13,
    o_C16_13: u64,
    c_C15_17: C15_17,
    o_C15_17: u64,
    c_C15_11: C15_11,
    o_C15_11: u64,
    c_C15_19: C15_19,
    o_C15_19: u64,
    c_C16_18: C16_18,
    o_C16_18: u64,
    c_C15_7: C15_7,
    o_C15_7: u64,
    c_C15_4: C15_4,
    o_C15_4: u64,
    c_C16_12: C16_12,
    o_C16_12: u64,
    c_C17_16: C17_16,
    o_C17_16: u64,
    c_C15_6: C15_6,
    o_C15_6: u64,
    c_C15_13: C15_13,
    o_C15_13: u64,
    c_C15_8: C15_8,
    o_C15_8: u64,
    c_C16_14: C16_14,
    o_C16_14: u64,
    c_C15_12: C15_12,
    o_C15_12: u64,
    c_C16_0: C16_0,
    o_C16_0: u64,
    c_C14_14: C14_14,
    o_C14_14: u64,
    c_C15_2: C15_2,
    o_C15_2: u64,
    c_C16_17: C16_17,
    o_C16_17: u64,
    c_C17_6: C17_6,
    o_C17_6: u64,
    c_C14_5: C14_5,
    o_C14_5: u64,
    c_C15_15: C15_15,
    o_C15_15: u64,
    c_C16_10: C16_10,
    o_C16_10: u64,
    c_C16_3: C16_3,
    o_C16_3: u64,
    c_C16_16: C16_16,
    o_C16_16: u64,
    c_C16_5: C16_5,
    o_C16_5: u64,
    c_C17_19: C17_19,
    o_C17_19: u64,
    c_C16_9: C16_9,
    o_C16_9: u64,
    c_C15_10: C15_10,
    o_C15_10: u64,
    c_C16_4: C16_4,
    o_C16_4: u64,
    c_C17_2: C17_2,
    o_C17_2: u64,
    c_C18_18: C18_18,
    o_C18_18: u64,
    c_C16_7: C16_7,
    o_C16_7: u64,
    c_C16_1: C16_1,
    o_C16_1: u64,
    c_C17_18: C17_18,
    o_C17_18: u64,
    c_C16_2: C16_2,
    o_C16_2: u64,
    c_C17_0: C17_0,
    o_C17_0: u64,
    c_C16_11: C16_11,
    o_C16_11: u64,
    c_C16_8: C16_8,
    o_C16_8: u64,
    c_C17_15: C17_15,
    o_C17_15: u64,
    c_C18_1: C18_1,
    o_C18_1: u64,
    c_C17_14: C17_14,
    o_C17_14: u64,
    c_C16_15: C16_15,
    o_C16_15: u64,
    c_C17_12: C17_12,
    o_C17_12: u64,
    c_C17_7: C17_7,
    o_C17_7: u64,
    c_C18_12: C18_12,
    o_C18_12: u64,
    c_C17_5: C17_5,
    o_C17_5: u64,
    c_C18_2: C18_2,
    o_C18_2: u64,
    c_C19_0: C19_0,
    o_C19_0: u64,
    c_C17_13: C17_13,
    o_C17_13: u64,
    c_C16_6: C16_6,
    o_C16_6: u64,
    c_C17_1: C17_1,
    o_C17_1: u64,
    c_C18_17: C18_17,
    o_C18_17: u64,
    c_C17_3: C17_3,
    o_C17_3: u64,
    c_C17_10: C17_10,
    o_C17_10: u64,
    c_C17_17: C17_17,
    o_C17_17: u64,
    c_C18_3: C18_3,
    o_C18_3: u64,
    c_C18_15: C18_15,
    o_C18_15: u64,
    c_C19_1: C19_1,
    o_C19_1: u64,
    c_C18_16: C18_16,
    o_C18_16: u64,
    c_C17_8: C17_8,
    o_C17_8: u64,
    c_C17_4: C17_4,
    o_C17_4: u64,
    c_C18_7: C18_7,
    o_C18_7: u64,
    c_C18_0: C18_0,
    o_C18_0: u64,
    c_C19_2: C19_2,
    o_C19_2: u64,
    c_C18_9: C18_9,
    o_C18_9: u64,
    c_C18_14: C18_14,
    o_C18_14: u64,
    c_C18_8: C18_8,
    o_C18_8: u64,
    c_C18_13: C18_13,
    o_C18_13: u64,
    c_C19_3: C19_3,
    o_C19_3: u64,
    c_C18_5: C18_5,
    o_C18_5: u64,
    c_C17_11: C17_11,
    o_C17_11: u64,
    c_C18_4: C18_4,
    o_C18_4: u64,
    c_C17_9: C17_9,
    o_C17_9: u64,
    c_C18_10: C18_10,
    o_C18_10: u64,
    c_C19_4: C19_4,
    o_C19_4: u64,
    c_C19_5: C19_5,
    o_C19_5: u64,
    c_C18_19: C18_19,
    o_C18_19: u64,
    c_C19_6: C19_6,
    o_C19_6: u64,
    c_C18_6: C18_6,
    o_C18_6: u64,
    c_C18_11: C18_11,
    o_C18_11: u64,
    c_C19_7: C19_7,
    o_C19_7: u64,
    c_C19_8: C19_8,
    o_C19_8: u64,
    c_C19_9: C19_9,
    o_C19_9: u64,
    c_C19_10: C19_10,
    o_C19_10: u64,
    c_C19_11: C19_11,
    o_C19_11: u64,
    c_C19_12: C19_12,
    o_C19_12: u64,
    c_C19_13: C19_13,
    o_C19_13: u64,
    c_C19_14: C19_14,
    o_C19_14: u64,
    c_C19_15: C19_15,
    o_C19_15: u64,
    c_C19_16: C19_16,
    o_C19_16: u64,
    c_C19_17: C19_17,
    o_C19_17: u64,
    c_C19_18: C19_18,
    o_C19_18: u64,
    c_C19_19: C19_19,
    o_C19_19: u64,
}
impl Hand {
    fn new() -> Self { Hand {
        c_C0_12: C0_12::new(),
        o_C0_12: 0,
        c_C0_18: C0_18::new(),
        o_C0_18: 0,
        c_C0_1: C0_1::new(),
        o_C0_1: 0,
        c_C0_15: C0_15::new(),
        o_C0_15: 0,
        c_C1_14: C1_14::new(),
        o_C1_14: 0,
        c_C0_9: C0_9::new(),
        o_C0_9: 0,
        c_C0_0: C0_0::new(),
        o_C0_0: 0,
        c_C0_13: C0_13::new(),
        o_C0_13: 0,
        c_C0_3: C0_3::new(),
        o_C0_3: 0,
        c_C1_10: C1_10::new(),
        o_C1_10: 0,
        c_C0_7: C0_7::new(),
        o_C0_7: 0,
        c_C0_5: C0_5::new(),
        o_C0_5: 0,
        c_C1_15: C1_15::new(),
        o_C1_15: 0,
        c_C0_14: C0_14::new(),
        o_C0_14: 0,
        c_C0_16: C0_16::new(),
        o_C0_16: 0,
        c_C1_17: C1_17::new(),
        o_C1_17: 0,
        c_C2_18: C2_18::new(),
        o_C2_18: 0,
        c_C0_4: C0_4::new(),
        o_C0_4: 0,
        c_C0_2: C0_2::new(),
        o_C0_2: 0,
        c_C0_8: C0_8::new(),
        o_C0_8: 0,
        c_C1_0: C1_0::new(),
        o_C1_0: 0,
        c_C1_7: C1_7::new(),
        o_C1_7: 0,
        c_C0_10: C0_10::new(),
        o_C0_10: 0,
        c_C0_17: C0_17::new(),
        o_C0_17: 0,
        c_C1_5: C1_5::new(),
        o_C1_5: 0,
        c_C2_10: C2_10::new(),
        o_C2_10: 0,
        c_C0_6: C0_6::new(),
        o_C0_6: 0,
        c_C1_2: C1_2::new(),
        o_C1_2: 0,
        c_C0_11: C0_11::new(),
        o_C0_11: 0,
        c_C1_18: C1_18::new(),
        o_C1_18: 0,
        c_C2_13: C2_13::new(),
        o_C2_13: 0,
        c_C0_19: C0_19::new(),
        o_C0_19: 0,
        c_C1_16: C1_16::new(),
        o_C1_16: 0,
        c_C2_1: C2_1::new(),
        o_C2_1: 0,
        c_C3_7: C3_7::new(),
        o_C3_7: 0,
        c_C1_13: C1_13::new(),
        o_C1_13: 0,
        c_C1_11: C1_11::new(),
        o_C1_11: 0,
        c_C2_9: C2_9::new(),
        o_C2_9: 0,
        c_C1_19: C1_19::new(),
        o_C1_19: 0,
        c_C1_12: C1_12::new(),
        o_C1_12: 0,
        c_C2_0: C2_0::new(),
        o_C2_0: 0,
        c_C1_4: C1_4::new(),
        o_C1_4: 0,
        c_C2_5: C2_5::new(),
        o_C2_5: 0,
        c_C1_6: C1_6::new(),
        o_C1_6: 0,
        c_C1_1: C1_1::new(),
        o_C1_1: 0,
        c_C2_6: C2_6::new(),
        o_C2_6: 0,
        c_C3_18: C3_18::new(),
        o_C3_18: 0,
        c_C1_3: C1_3::new(),
        o_C1_3: 0,
        c_C1_9: C1_9::new(),
        o_C1_9: 0,
        c_C2_19: C2_19::new(),
        o_C2_19: 0,
        c_C1_8: C1_8::new(),
        o_C1_8: 0,
        c_C2_16: C2_16::new(),
        o_C2_16: 0,
        c_C2_12: C2_12::new(),
        o_C2_12: 0,
        c_C3_13: C3_13::new(),
        o_C3_13: 0,
        c_C2_17: C2_17::new(),
        o_C2_17: 0,
        c_C2_7: C2_7::new(),
        o_C2_7: 0,
        c_C2_14: C2_14::new(),
        o_C2_14: 0,
        c_C3_5: C3_5::new(),
        o_C3_5: 0,
        c_C4_16: C4_16::new(),
        o_C4_16: 0,
        c_C2_4: C2_4::new(),
        o_C2_4: 0,
        c_C3_19: C3_19::new(),
        o_C3_19: 0,
        c_C2_3: C2_3::new(),
        o_C2_3: 0,
        c_C3_16: C3_16::new(),
        o_C3_16: 0,
        c_C3_8: C3_8::new(),
        o_C3_8: 0,
        c_C3_11: C3_11::new(),
        o_C3_11: 0,
        c_C4_5: C4_5::new(),
        o_C4_5: 0,
        c_C3_3: C3_3::new(),
        o_C3_3: 0,
        c_C3_12: C3_12::new(),
        o_C3_12: 0,
        c_C4_18: C4_18::new(),
        o_C4_18: 0,
        c_C2_15: C2_15::new(),
        o_C2_15: 0,
        c_C3_17: C3_17::new(),
        o_C3_17: 0,
        c_C3_6: C3_6::new(),
        o_C3_6: 0,
        c_C2_8: C2_8::new(),
        o_C2_8: 0,
        c_C3_10: C3_10::new(),
        o_C3_10: 0,
        c_C4_4: C4_4::new(),
        o_C4_4: 0,
        c_C5_18: C5_18::new(),
        o_C5_18: 0,
        c_C3_2: C3_2::new(),
        o_C3_2: 0,
        c_C4_10: C4_10::new(),
        o_C4_10: 0,
        c_C2_2: C2_2::new(),
        o_C2_2: 0,
        c_C3_9: C3_9::new(),
        o_C3_9: 0,
        c_C3_0: C3_0::new(),
        o_C3_0: 0,
        c_C4_14: C4_14::new(),
        o_C4_14: 0,
        c_C3_4: C3_4::new(),
        o_C3_4: 0,
        c_C4_3: C4_3::new(),
        o_C4_3: 0,
        c_C3_1: C3_1::new(),
        o_C3_1: 0,
        c_C4_8: C4_8::new(),
        o_C4_8: 0,
        c_C5_4: C5_4::new(),
        o_C5_4: 0,
        c_C4_9: C4_9::new(),
        o_C4_9: 0,
        c_C5_19: C5_19::new(),
        o_C5_19: 0,
        c_C4_2: C4_2::new(),
        o_C4_2: 0,
        c_C5_17: C5_17::new(),
        o_C5_17: 0,
        c_C6_10: C6_10::new(),
        o_C6_10: 0,
        c_C4_11: C4_11::new(),
        o_C4_11: 0,
        c_C4_15: C4_15::new(),
        o_C4_15: 0,
        c_C5_16: C5_16::new(),
        o_C5_16: 0,
        c_C4_12: C4_12::new(),
        o_C4_12: 0,
        c_C5_14: C5_14::new(),
        o_C5_14: 0,
        c_C4_6: C4_6::new(),
        o_C4_6: 0,
        c_C3_15: C3_15::new(),
        o_C3_15: 0,
        c_C4_7: C4_7::new(),
        o_C4_7: 0,
        c_C5_7: C5_7::new(),
        o_C5_7: 0,
        c_C6_14: C6_14::new(),
        o_C6_14: 0,
        c_C4_1: C4_1::new(),
        o_C4_1: 0,
        c_C4_0: C4_0::new(),
        o_C4_0: 0,
        c_C2_11: C2_11::new(),
        o_C2_11: 0,
        c_C3_14: C3_14::new(),
        o_C3_14: 0,
        c_C4_17: C4_17::new(),
        o_C4_17: 0,
        c_C5_1: C5_1::new(),
        o_C5_1: 0,
        c_C4_19: C4_19::new(),
        o_C4_19: 0,
        c_C5_8: C5_8::new(),
        o_C5_8: 0,
        c_C5_2: C5_2::new(),
        o_C5_2: 0,
        c_C6_4: C6_4::new(),
        o_C6_4: 0,
        c_C5_10: C5_10::new(),
        o_C5_10: 0,
        c_C5_15: C5_15::new(),
        o_C5_15: 0,
        c_C6_15: C6_15::new(),
        o_C6_15: 0,
        c_C7_9: C7_9::new(),
        o_C7_9: 0,
        c_C5_13: C5_13::new(),
        o_C5_13: 0,
        c_C6_5: C6_5::new(),
        o_C6_5: 0,
        c_C6_17: C6_17::new(),
        o_C6_17: 0,
        c_C5_12: C5_12::new(),
        o_C5_12: 0,
        c_C6_11: C6_11::new(),
        o_C6_11: 0,
        c_C7_17: C7_17::new(),
        o_C7_17: 0,
        c_C5_5: C5_5::new(),
        o_C5_5: 0,
        c_C6_12: C6_12::new(),
        o_C6_12: 0,
        c_C5_9: C5_9::new(),
        o_C5_9: 0,
        c_C5_6: C5_6::new(),
        o_C5_6: 0,
        c_C6_0: C6_0::new(),
        o_C6_0: 0,
        c_C6_6: C6_6::new(),
        o_C6_6: 0,
        c_C7_12: C7_12::new(),
        o_C7_12: 0,
        c_C6_3: C6_3::new(),
        o_C6_3: 0,
        c_C5_0: C5_0::new(),
        o_C5_0: 0,
        c_C6_13: C6_13::new(),
        o_C6_13: 0,
        c_C7_10: C7_10::new(),
        o_C7_10: 0,
        c_C8_5: C8_5::new(),
        o_C8_5: 0,
        c_C5_11: C5_11::new(),
        o_C5_11: 0,
        c_C6_9: C6_9::new(),
        o_C6_9: 0,
        c_C7_19: C7_19::new(),
        o_C7_19: 0,
        c_C6_8: C6_8::new(),
        o_C6_8: 0,
        c_C6_19: C6_19::new(),
        o_C6_19: 0,
        c_C7_16: C7_16::new(),
        o_C7_16: 0,
        c_C6_16: C6_16::new(),
        o_C6_16: 0,
        c_C7_13: C7_13::new(),
        o_C7_13: 0,
        c_C8_9: C8_9::new(),
        o_C8_9: 0,
        c_C7_4: C7_4::new(),
        o_C7_4: 0,
        c_C8_17: C8_17::new(),
        o_C8_17: 0,
        c_C7_8: C7_8::new(),
        o_C7_8: 0,
        c_C6_2: C6_2::new(),
        o_C6_2: 0,
        c_C7_6: C7_6::new(),
        o_C7_6: 0,
        c_C6_7: C6_7::new(),
        o_C6_7: 0,
        c_C7_14: C7_14::new(),
        o_C7_14: 0,
        c_C8_8: C8_8::new(),
        o_C8_8: 0,
        c_C9_7: C9_7::new(),
        o_C9_7: 0,
        c_C7_0: C7_0::new(),
        o_C7_0: 0,
        c_C8_7: C8_7::new(),
        o_C8_7: 0,
        c_C7_18: C7_18::new(),
        o_C7_18: 0,
        c_C7_15: C7_15::new(),
        o_C7_15: 0,
        c_C6_18: C6_18::new(),
        o_C6_18: 0,
        c_C6_1: C6_1::new(),
        o_C6_1: 0,
        c_C7_3: C7_3::new(),
        o_C7_3: 0,
        c_C8_6: C8_6::new(),
        o_C8_6: 0,
        c_C8_2: C8_2::new(),
        o_C8_2: 0,
        c_C9_12: C9_12::new(),
        o_C9_12: 0,
        c_C7_5: C7_5::new(),
        o_C7_5: 0,
        c_C8_15: C8_15::new(),
        o_C8_15: 0,
        c_C7_11: C7_11::new(),
        o_C7_11: 0,
        c_C8_3: C8_3::new(),
        o_C8_3: 0,
        c_C7_2: C7_2::new(),
        o_C7_2: 0,
        c_C8_1: C8_1::new(),
        o_C8_1: 0,
        c_C9_19: C9_19::new(),
        o_C9_19: 0,
        c_C8_4: C8_4::new(),
        o_C8_4: 0,
        c_C8_18: C8_18::new(),
        o_C8_18: 0,
        c_C7_7: C7_7::new(),
        o_C7_7: 0,
        c_C8_14: C8_14::new(),
        o_C8_14: 0,
        c_C9_15: C9_15::new(),
        o_C9_15: 0,
        c_C10_3: C10_3::new(),
        o_C10_3: 0,
        c_C8_11: C8_11::new(),
        o_C8_11: 0,
        c_C8_16: C8_16::new(),
        o_C8_16: 0,
        c_C9_6: C9_6::new(),
        o_C9_6: 0,
        c_C9_5: C9_5::new(),
        o_C9_5: 0,
        c_C8_10: C8_10::new(),
        o_C8_10: 0,
        c_C9_4: C9_4::new(),
        o_C9_4: 0,
        c_C10_10: C10_10::new(),
        o_C10_10: 0,
        c_C8_12: C8_12::new(),
        o_C8_12: 0,
        c_C9_16: C9_16::new(),
        o_C9_16: 0,
        c_C7_1: C7_1::new(),
        o_C7_1: 0,
        c_C8_19: C8_19::new(),
        o_C8_19: 0,
        c_C9_14: C9_14::new(),
        o_C9_14: 0,
        c_C8_0: C8_0::new(),
        o_C8_0: 0,
        c_C9_10: C9_10::new(),
        o_C9_10: 0,
        c_C10_1: C10_1::new(),
        o_C10_1: 0,
        c_C10_4: C10_4::new(),
        o_C10_4: 0,
        c_C11_8: C11_8::new(),
        o_C11_8: 0,
        c_C9_2: C9_2::new(),
        o_C9_2: 0,
        c_C9_9: C9_9::new(),
        o_C9_9: 0,
        c_C10_13: C10_13::new(),
        o_C10_13: 0,
        c_C9_18: C9_18::new(),
        o_C9_18: 0,
        c_C9_3: C9_3::new(),
        o_C9_3: 0,
        c_C10_7: C10_7::new(),
        o_C10_7: 0,
        c_C9_13: C9_13::new(),
        o_C9_13: 0,
        c_C10_16: C10_16::new(),
        o_C10_16: 0,
        c_C11_15: C11_15::new(),
        o_C11_15: 0,
        c_C9_1: C9_1::new(),
        o_C9_1: 0,
        c_C8_13: C8_13::new(),
        o_C8_13: 0,
        c_C9_0: C9_0::new(),
        o_C9_0: 0,
        c_C9_17: C9_17::new(),
        o_C9_17: 0,
        c_C10_8: C10_8::new(),
        o_C10_8: 0,
        c_C9_8: C9_8::new(),
        o_C9_8: 0,
        c_C10_12: C10_12::new(),
        o_C10_12: 0,
        c_C10_9: C10_9::new(),
        o_C10_9: 0,
        c_C11_14: C11_14::new(),
        o_C11_14: 0,
        c_C10_2: C10_2::new(),
        o_C10_2: 0,
        c_C10_19: C10_19::new(),
        o_C10_19: 0,
        c_C11_11: C11_11::new(),
        o_C11_11: 0,
        c_C12_8: C12_8::new(),
        o_C12_8: 0,
        c_C10_17: C10_17::new(),
        o_C10_17: 0,
        c_C10_0: C10_0::new(),
        o_C10_0: 0,
        c_C10_5: C10_5::new(),
        o_C10_5: 0,
        c_C11_19: C11_19::new(),
        o_C11_19: 0,
        c_C10_14: C10_14::new(),
        o_C10_14: 0,
        c_C11_10: C11_10::new(),
        o_C11_10: 0,
        c_C10_15: C10_15::new(),
        o_C10_15: 0,
        c_C11_17: C11_17::new(),
        o_C11_17: 0,
        c_C10_6: C10_6::new(),
        o_C10_6: 0,
        c_C11_16: C11_16::new(),
        o_C11_16: 0,
        c_C12_13: C12_13::new(),
        o_C12_13: 0,
        c_C11_3: C11_3::new(),
        o_C11_3: 0,
        c_C11_4: C11_4::new(),
        o_C11_4: 0,
        c_C12_6: C12_6::new(),
        o_C12_6: 0,
        c_C10_18: C10_18::new(),
        o_C10_18: 0,
        c_C11_6: C11_6::new(),
        o_C11_6: 0,
        c_C11_12: C11_12::new(),
        o_C11_12: 0,
        c_C12_0: C12_0::new(),
        o_C12_0: 0,
        c_C13_4: C13_4::new(),
        o_C13_4: 0,
        c_C10_11: C10_11::new(),
        o_C10_11: 0,
        c_C11_2: C11_2::new(),
        o_C11_2: 0,
        c_C11_18: C11_18::new(),
        o_C11_18: 0,
        c_C12_18: C12_18::new(),
        o_C12_18: 0,
        c_C11_9: C11_9::new(),
        o_C11_9: 0,
        c_C12_4: C12_4::new(),
        o_C12_4: 0,
        c_C12_17: C12_17::new(),
        o_C12_17: 0,
        c_C11_0: C11_0::new(),
        o_C11_0: 0,
        c_C12_5: C12_5::new(),
        o_C12_5: 0,
        c_C13_13: C13_13::new(),
        o_C13_13: 0,
        c_C11_5: C11_5::new(),
        o_C11_5: 0,
        c_C12_14: C12_14::new(),
        o_C12_14: 0,
        c_C11_13: C11_13::new(),
        o_C11_13: 0,
        c_C12_3: C12_3::new(),
        o_C12_3: 0,
        c_C11_7: C11_7::new(),
        o_C11_7: 0,
        c_C12_15: C12_15::new(),
        o_C12_15: 0,
        c_C12_11: C12_11::new(),
        o_C12_11: 0,
        c_C13_19: C13_19::new(),
        o_C13_19: 0,
        c_C13_0: C13_0::new(),
        o_C13_0: 0,
        c_C14_0: C14_0::new(),
        o_C14_0: 0,
        c_C12_12: C12_12::new(),
        o_C12_12: 0,
        c_C13_17: C13_17::new(),
        o_C13_17: 0,
        c_C11_1: C11_1::new(),
        o_C11_1: 0,
        c_C12_9: C12_9::new(),
        o_C12_9: 0,
        c_C12_16: C12_16::new(),
        o_C12_16: 0,
        c_C12_10: C12_10::new(),
        o_C12_10: 0,
        c_C12_19: C12_19::new(),
        o_C12_19: 0,
        c_C13_9: C13_9::new(),
        o_C13_9: 0,
        c_C13_15: C13_15::new(),
        o_C13_15: 0,
        c_C12_2: C12_2::new(),
        o_C12_2: 0,
        c_C13_6: C13_6::new(),
        o_C13_6: 0,
        c_C14_19: C14_19::new(),
        o_C14_19: 0,
        c_C13_5: C13_5::new(),
        o_C13_5: 0,
        c_C12_7: C12_7::new(),
        o_C12_7: 0,
        c_C13_2: C13_2::new(),
        o_C13_2: 0,
        c_C14_9: C14_9::new(),
        o_C14_9: 0,
        c_C12_1: C12_1::new(),
        o_C12_1: 0,
        c_C13_7: C13_7::new(),
        o_C13_7: 0,
        c_C13_3: C13_3::new(),
        o_C13_3: 0,
        c_C13_8: C13_8::new(),
        o_C13_8: 0,
        c_C14_7: C14_7::new(),
        o_C14_7: 0,
        c_C15_18: C15_18::new(),
        o_C15_18: 0,
        c_C13_1: C13_1::new(),
        o_C13_1: 0,
        c_C14_10: C14_10::new(),
        o_C14_10: 0,
        c_C13_14: C13_14::new(),
        o_C13_14: 0,
        c_C13_16: C13_16::new(),
        o_C13_16: 0,
        c_C14_15: C14_15::new(),
        o_C14_15: 0,
        c_C14_3: C14_3::new(),
        o_C14_3: 0,
        c_C15_0: C15_0::new(),
        o_C15_0: 0,
        c_C13_11: C13_11::new(),
        o_C13_11: 0,
        c_C14_11: C14_11::new(),
        o_C14_11: 0,
        c_C15_9: C15_9::new(),
        o_C15_9: 0,
        c_C13_10: C13_10::new(),
        o_C13_10: 0,
        c_C14_4: C14_4::new(),
        o_C14_4: 0,
        c_C13_12: C13_12::new(),
        o_C13_12: 0,
        c_C14_8: C14_8::new(),
        o_C14_8: 0,
        c_C13_18: C13_18::new(),
        o_C13_18: 0,
        c_C14_2: C14_2::new(),
        o_C14_2: 0,
        c_C15_14: C15_14::new(),
        o_C15_14: 0,
        c_C16_19: C16_19::new(),
        o_C16_19: 0,
        c_C14_1: C14_1::new(),
        o_C14_1: 0,
        c_C15_1: C15_1::new(),
        o_C15_1: 0,
        c_C14_18: C14_18::new(),
        o_C14_18: 0,
        c_C14_6: C14_6::new(),
        o_C14_6: 0,
        c_C14_16: C14_16::new(),
        o_C14_16: 0,
        c_C15_16: C15_16::new(),
        o_C15_16: 0,
        c_C14_17: C14_17::new(),
        o_C14_17: 0,
        c_C15_3: C15_3::new(),
        o_C15_3: 0,
        c_C14_13: C14_13::new(),
        o_C14_13: 0,
        c_C15_5: C15_5::new(),
        o_C15_5: 0,
        c_C16_13: C16_13::new(),
        o_C16_13: 0,
        c_C15_17: C15_17::new(),
        o_C15_17: 0,
        c_C15_11: C15_11::new(),
        o_C15_11: 0,
        c_C15_19: C15_19::new(),
        o_C15_19: 0,
        c_C16_18: C16_18::new(),
        o_C16_18: 0,
        c_C15_7: C15_7::new(),
        o_C15_7: 0,
        c_C15_4: C15_4::new(),
        o_C15_4: 0,
        c_C16_12: C16_12::new(),
        o_C16_12: 0,
        c_C17_16: C17_16::new(),
        o_C17_16: 0,
        c_C15_6: C15_6::new(),
        o_C15_6: 0,
        c_C15_13: C15_13::new(),
        o_C15_13: 0,
        c_C15_8: C15_8::new(),
        o_C15_8: 0,
        c_C16_14: C16_14::new(),
        o_C16_14: 0,
        c_C15_12: C15_12::new(),
        o_C15_12: 0,
        c_C16_0: C16_0::new(),
        o_C16_0: 0,
        c_C14_14: C14_14::new(),
        o_C14_14: 0,
        c_C15_2: C15_2::new(),
        o_C15_2: 0,
        c_C16_17: C16_17::new(),
        o_C16_17: 0,
        c_C17_6: C17_6::new(),
        o_C17_6: 0,
        c_C14_5: C14_5::new(),
        o_C14_5: 0,
        c_C15_15: C15_15::new(),
        o_C15_15: 0,
        c_C16_10: C16_10::new(),
        o_C16_10: 0,
        c_C16_3: C16_3::new(),
        o_C16_3: 0,
        c_C16_16: C16_16::new(),
        o_C16_16: 0,
        c_C16_5: C16_5::new(),
        o_C16_5: 0,
        c_C17_19: C17_19::new(),
        o_C17_19: 0,
        c_C16_9: C16_9::new(),
        o_C16_9: 0,
        c_C15_10: C15_10::new(),
        o_C15_10: 0,
        c_C16_4: C16_4::new(),
        o_C16_4: 0,
        c_C17_2: C17_2::new(),
        o_C17_2: 0,
        c_C18_18: C18_18::new(),
        o_C18_18: 0,
        c_C16_7: C16_7::new(),
        o_C16_7: 0,
        c_C16_1: C16_1::new(),
        o_C16_1: 0,
        c_C17_18: C17_18::new(),
        o_C17_18: 0,
        c_C16_2: C16_2::new(),
        o_C16_2: 0,
        c_C17_0: C17_0::new(),
        o_C17_0: 0,
        c_C16_11: C16_11::new(),
        o_C16_11: 0,
        c_C16_8: C16_8::new(),
        o_C16_8: 0,
        c_C17_15: C17_15::new(),
        o_C17_15: 0,
        c_C18_1: C18_1::new(),
        o_C18_1: 0,
        c_C17_14: C17_14::new(),
        o_C17_14: 0,
        c_C16_15: C16_15::new(),
        o_C16_15: 0,
        c_C17_12: C17_12::new(),
        o_C17_12: 0,
        c_C17_7: C17_7::new(),
        o_C17_7: 0,
        c_C18_12: C18_12::new(),
        o_C18_12: 0,
        c_C17_5: C17_5::new(),
        o_C17_5: 0,
        c_C18_2: C18_2::new(),
        o_C18_2: 0,
        c_C19_0: C19_0::new(),
        o_C19_0: 0,
        c_C17_13: C17_13::new(),
        o_C17_13: 0,
        c_C16_6: C16_6::new(),
        o_C16_6: 0,
        c_C17_1: C17_1::new(),
        o_C17_1: 0,
        c_C18_17: C18_17::new(),
        o_C18_17: 0,
        c_C17_3: C17_3::new(),
        o_C17_3: 0,
        c_C17_10: C17_10::new(),
        o_C17_10: 0,
        c_C17_17: C17_17::new(),
        o_C17_17: 0,
        c_C18_3: C18_3::new(),
        o_C18_3: 0,
        c_C18_15: C18_15::new(),
        o_C18_15: 0,
        c_C19_1: C19_1::new(),
        o_C19_1: 0,
        c_C18_16: C18_16::new(),
        o_C18_16: 0,
        c_C17_8: C17_8::new(),
        o_C17_8: 0,
        c_C17_4: C17_4::new(),
        o_C17_4: 0,
        c_C18_7: C18_7::new(),
        o_C18_7: 0,
        c_C18_0: C18_0::new(),
        o_C18_0: 0,
        c_C19_2: C19_2::new(),
        o_C19_2: 0,
        c_C18_9: C18_9::new(),
        o_C18_9: 0,
        c_C18_14: C18_14::new(),
        o_C18_14: 0,
        c_C18_8: C18_8::new(),
        o_C18_8: 0,
        c_C18_13: C18_13::new(),
        o_C18_13: 0,
        c_C19_3: C19_3::new(),
        o_C19_3: 0,
        c_C18_5: C18_5::new(),
        o_C18_5: 0,
        c_C17_11: C17_11::new(),
        o_C17_11: 0,
        c_C18_4: C18_4::new(),
        o_C18_4: 0,
        c_C17_9: C17_9::new(),
        o_C17_9: 0,
        c_C18_10: C18_10::new(),
        o_C18_10: 0,
        c_C19_4: C19_4::new(),
        o_C19_4: 0,
        c_C19_5: C19_5::new(),
        o_C19_5: 0,
        c_C18_19: C18_19::new(),
        o_C18_19: 0,
        c_C19_6: C19_6::new(),
        o_C19_6: 0,
        c_C18_6: C18_6::new(),
        o_C18_6: 0,
        c_C18_11: C18_11::new(),
        o_C18_11: 0,
        c_C19_7: C19_7::new(),
        o_C19_7: 0,
        c_C19_8: C19_8::new(),
        o_C19_8: 0,
        c_C19_9: C19_9::new(),
        o_C19_9: 0,
        c_C19_10: C19_10::new(),
        o_C19_10: 0,
        c_C19_11: C19_11::new(),
        o_C19_11: 0,
        c_C19_12: C19_12::new(),
        o_C19_12: 0,
        c_C19_13: C19_13::new(),
        o_C19_13: 0,
        c_C19_14: C19_14::new(),
        o_C19_14: 0,
        c_C19_15: C19_15::new(),
        o_C19_15: 0,
        c_C19_16: C19_16::new(),
        o_C19_16: 0,
        c_C19_17: C19_17::new(),
        o_C19_17: 0,
        c_C19_18: C19_18::new(),
        o_C19_18: 0,
        c_C19_19: C19_19::new(),
        o_C19_19: 0,
    } }
    #[inline(never)]
    fn run(&mut self) -> u64 {
        self.o_C0_12 = self.c_C0_12.update();
        self.o_C0_18 = self.c_C0_18.update();
        self.o_C0_1 = self.c_C0_1.update();
        self.o_C0_15 = self.c_C0_15.update();
        self.o_C1_14 = self.c_C1_14.update(&self.o_C0_12, &self.o_C0_18, &self.o_C0_1, &self.o_C0_15);
        self.o_C0_9 = self.c_C0_9.update();
        self.o_C0_0 = self.c_C0_0.update();
        self.o_C0_13 = self.c_C0_13.update();
        self.o_C0_3 = self.c_C0_3.update();
        self.o_C1_10 = self.c_C1_10.update(&self.o_C0_9, &self.o_C0_0, &self.o_C0_13, &self.o_C0_3);
        self.o_C0_7 = self.c_C0_7.update();
        self.o_C0_5 = self.c_C0_5.update();
        self.o_C1_15 = self.c_C1_15.update(&self.o_C0_7, &self.o_C0_12, &self.o_C0_13, &self.o_C0_5);
        self.o_C0_14 = self.c_C0_14.update();
        self.o_C0_16 = self.c_C0_16.update();
        self.o_C1_17 = self.c_C1_17.update(&self.o_C0_14, &self.o_C0_16, &self.o_C0_3, &self.o_C0_5);
        self.o_C2_18 = self.c_C2_18.update(&self.o_C1_14, &self.o_C1_10, &self.o_C1_15, &self.o_C1_17);
        self.o_C0_4 = self.c_C0_4.update();
        self.o_C0_2 = self.c_C0_2.update();
        self.o_C0_8 = self.c_C0_8.update();
        self.o_C1_0 = self.c_C1_0.update(&self.o_C0_4, &self.o_C0_18, &self.o_C0_2, &self.o_C0_8);
        self.o_C1_7 = self.c_C1_7.update(&self.o_C0_13, &self.o_C0_0, &self.o_C0_16, &self.o_C0_7);
        self.o_C0_10 = self.c_C0_10.update();
        self.o_C0_17 = self.c_C0_17.update();
        self.o_C1_5 = self.c_C1_5.update(&self.o_C0_10, &self.o_C0_0, &self.o_C0_18, &self.o_C0_17);
        self.o_C2_10 = self.c_C2_10.update(&self.o_C1_14, &self.o_C1_0, &self.o_C1_7, &self.o_C1_5);
        self.o_C0_6 = self.c_C0_6.update();
        self.o_C1_2 = self.c_C1_2.update(&self.o_C0_12, &self.o_C0_6, &self.o_C0_3, &self.o_C0_15);
        self.o_C0_11 = self.c_C0_11.update();
        self.o_C1_18 = self.c_C1_18.update(&self.o_C0_16, &self.o_C0_12, &self.o_C0_11, &self.o_C0_15);
        self.o_C2_13 = self.c_C2_13.update(&self.o_C1_2, &self.o_C1_0, &self.o_C1_14, &self.o_C1_18);
        self.o_C0_19 = self.c_C0_19.update();
        self.o_C1_16 = self.c_C1_16.update(&self.o_C0_11, &self.o_C0_17, &self.o_C0_19, &self.o_C0_2);
        self.o_C2_1 = self.c_C2_1.update(&self.o_C1_5, &self.o_C1_16, &self.o_C1_7, &self.o_C1_0);
        self.o_C3_7 = self.c_C3_7.update(&self.o_C2_18, &self.o_C2_10, &self.o_C2_13, &self.o_C2_1);
        self.o_C1_13 = self.c_C1_13.update(&self.o_C0_9, &self.o_C0_19, &self.o_C0_15, &self.o_C0_16);
        self.o_C1_11 = self.c_C1_11.update(&self.o_C0_5, &self.o_C0_9, &self.o_C0_3, &self.o_C0_10);
        self.o_C2_9 = self.c_C2_9.update(&self.o_C1_13, &self.o_C1_11, &self.o_C1_0, &self.o_C1_10);
        self.o_C1_19 = self.c_C1_19.update(&self.o_C0_0, &self.o_C0_15, &self.o_C0_1, &self.o_C0_9);
        self.o_C1_12 = self.c_C1_12.update(&self.o_C0_16, &self.o_C0_13, &self.o_C0_19, &self.o_C0_6);
        self.o_C2_0 = self.c_C2_0.update(&self.o_C1_19, &self.o_C1_18, &self.o_C1_12, &self.o_C1_5);
        self.o_C1_4 = self.c_C1_4.update(&self.o_C0_14, &self.o_C0_8, &self.o_C0_7, &self.o_C0_3);
        self.o_C2_5 = self.c_C2_5.update(&self.o_C1_12, &self.o_C1_16, &self.o_C1_4, &self.o_C1_18);
        self.o_C1_6 = self.c_C1_6.update(&self.o_C0_17, &self.o_C0_0, &self.o_C0_12, &self.o_C0_6);
        self.o_C1_1 = self.c_C1_1.update(&self.o_C0_3, &self.o_C0_15, &self.o_C0_14, &self.o_C0_18);
        self.o_C2_6 = self.c_C2_6.update(&self.o_C1_17, &self.o_C1_6, &self.o_C1_13, &self.o_C1_1);
        self.o_C3_18 = self.c_C3_18.update(&self.o_C2_9, &self.o_C2_0, &self.o_C2_5, &self.o_C2_6);
        self.o_C1_3 = self.c_C1_3.update(&self.o_C0_0, &self.o_C0_12, &self.o_C0_13, &self.o_C0_19);
        self.o_C1_9 = self.c_C1_9.update(&self.o_C0_11, &self.o_C0_7, &self.o_C0_18, &self.o_C0_14);
        self.o_C2_19 = self.c_C2_19.update(&self.o_C1_3, &self.o_C1_0, &self.o_C1_9, &self.o_C1_12);
        self.o_C1_8 = self.c_C1_8.update(&self.o_C0_14, &self.o_C0_15, &self.o_C0_17, &self.o_C0_7);
        self.o_C2_16 = self.c_C2_16.update(&self.o_C1_2, &self.o_C1_5, &self.o_C1_18, &self.o_C1_8);
        self.o_C2_12 = self.c_C2_12.update(&self.o_C1_17, &self.o_C1_8, &self.o_C1_1, &self.o_C1_2);
        self.o_C3_13 = self.c_C3_13.update(&self.o_C2_19, &self.o_C2_16, &self.o_C2_1, &self.o_C2_12);
        self.o_C2_17 = self.c_C2_17.update(&self.o_C1_16, &self.o_C1_5, &self.o_C1_8, &self.o_C1_9);
        self.o_C2_7 = self.c_C2_7.update(&self.o_C1_15, &self.o_C1_11, &self.o_C1_17, &self.o_C1_6);
        self.o_C2_14 = self.c_C2_14.update(&self.o_C1_8, &self.o_C1_7, &self.o_C1_19, &self.o_C1_3);
        self.o_C3_5 = self.c_C3_5.update(&self.o_C2_17, &self.o_C2_7, &self.o_C2_16, &self.o_C2_14);
        self.o_C4_16 = self.c_C4_16.update(&self.o_C3_7, &self.o_C3_18, &self.o_C3_13, &self.o_C3_5);
        self.o_C2_4 = self.c_C2_4.update(&self.o_C1_14, &self.o_C1_8, &self.o_C1_17, &self.o_C1_0);
        self.o_C3_19 = self.c_C3_19.update(&self.o_C2_10, &self.o_C2_18, &self.o_C2_4, &self.o_C2_19);
        self.o_C2_3 = self.c_C2_3.update(&self.o_C1_12, &self.o_C1_16, &self.o_C1_11, &self.o_C1_17);
        self.o_C3_16 = self.c_C3_16.update(&self.o_C2_3, &self.o_C2_12, &self.o_C2_9, &self.o_C2_16);
        self.o_C3_8 = self.c_C3_8.update(&self.o_C2_9, &self.o_C2_4, &self.o_C2_6, &self.o_C2_1);
        self.o_C3_11 = self.c_C3_11.update(&self.o_C2_4, &self.o_C2_0, &self.o_C2_17, &self.o_C2_1);
        self.o_C4_5 = self.c_C4_5.update(&self.o_C3_19, &self.o_C3_16, &self.o_C3_8, &self.o_C3_11);
        self.o_C3_3 = self.c_C3_3.update(&self.o_C2_0, &self.o_C2_12, &self.o_C2_4, &self.o_C2_1);
        self.o_C3_12 = self.c_C3_12.update(&self.o_C2_18, &self.o_C2_6, &self.o_C2_14, &self.o_C2_5);
        self.o_C4_18 = self.c_C4_18.update(&self.o_C3_5, &self.o_C3_3, &self.o_C3_13, &self.o_C3_12);
        self.o_C2_15 = self.c_C2_15.update(&self.o_C1_19, &self.o_C1_5, &self.o_C1_11, &self.o_C1_9);
        self.o_C3_17 = self.c_C3_17.update(&self.o_C2_15, &self.o_C2_0, &self.o_C2_10, &self.o_C2_12);
        self.o_C3_6 = self.c_C3_6.update(&self.o_C2_7, &self.o_C2_16, &self.o_C2_0, &self.o_C2_12);
        self.o_C2_8 = self.c_C2_8.update(&self.o_C1_16, &self.o_C1_13, &self.o_C1_15, &self.o_C1_11);
        self.o_C3_10 = self.c_C3_10.update(&self.o_C2_9, &self.o_C2_5, &self.o_C2_13, &self.o_C2_8);
        self.o_C4_4 = self.c_C4_4.update(&self.o_C3_17, &self.o_C3_6, &self.o_C3_8, &self.o_C3_10);
        self.o_C5_18 = self.c_C5_18.update(&self.o_C4_16, &self.o_C4_5, &self.o_C4_18, &self.o_C4_4);
        self.o_C3_2 = self.c_C3_2.update(&self.o_C2_19, &self.o_C2_13, &self.o_C2_0, &self.o_C2_7);
        self.o_C4_10 = self.c_C4_10.update(&self.o_C3_3, &self.o_C3_18, &self.o_C3_12, &self.o_C3_2);
        self.o_C2_2 = self.c_C2_2.update(&self.o_C1_6, &self.o_C1_17, &self.o_C1_18, &self.o_C1_7);
        self.o_C3_9 = self.c_C3_9.update(&self.o_C2_9, &self.o_C2_2, &self.o_C2_18, &self.o_C2_19);
        self.o_C3_0 = self.c_C3_0.update(&self.o_C2_10, &self.o_C2_13, &self.o_C2_6, &self.o_C2_8);
        self.o_C4_14 = self.c_C4_14.update(&self.o_C3_9, &self.o_C3_0, &self.o_C3_18, &self.o_C3_2);
        self.o_C3_4 = self.c_C3_4.update(&self.o_C2_5, &self.o_C2_14, &self.o_C2_16, &self.o_C2_13);
        self.o_C4_3 = self.c_C4_3.update(&self.o_C3_2, &self.o_C3_4, &self.o_C3_5, &self.o_C3_17);
        self.o_C3_1 = self.c_C3_1.update(&self.o_C2_3, &self.o_C2_8, &self.o_C2_16, &self.o_C2_6);
        self.o_C4_8 = self.c_C4_8.update(&self.o_C3_10, &self.o_C3_1, &self.o_C3_13, &self.o_C3_2);
        self.o_C5_4 = self.c_C5_4.update(&self.o_C4_10, &self.o_C4_14, &self.o_C4_3, &self.o_C4_8);
        self.o_C4_9 = self.c_C4_9.update(&self.o_C3_12, &self.o_C3_4, &self.o_C3_18, &self.o_C3_10);
        self.o_C5_19 = self.c_C5_19.update(&self.o_C4_4, &self.o_C4_10, &self.o_C4_9, &self.o_C4_3);
        self.o_C4_2 = self.c_C4_2.update(&self.o_C3_17, &self.o_C3_7, &self.o_C3_2, &self.o_C3_1);
        self.o_C5_17 = self.c_C5_17.update(&self.o_C4_3, &self.o_C4_16, &self.o_C4_10, &self.o_C4_2);
        self.o_C6_10 = self.c_C6_10.update(&self.o_C5_18, &self.o_C5_4, &self.o_C5_19, &self.o_C5_17);
        self.o_C4_11 = self.c_C4_11.update(&self.o_C3_18, &self.o_C3_17, &self.o_C3_7, &self.o_C3_2);
        self.o_C4_15 = self.c_C4_15.update(&self.o_C3_13, &self.o_C3_3, &self.o_C3_1, &self.o_C3_6);
        self.o_C5_16 = self.c_C5_16.update(&self.o_C4_11, &self.o_C4_15, &self.o_C4_18, &self.o_C4_4);
        self.o_C4_12 = self.c_C4_12.update(&self.o_C3_8, &self.o_C3_11, &self.o_C3_9, &self.o_C3_3);
        self.o_C5_14 = self.c_C5_14.update(&self.o_C4_12, &self.o_C4_2, &self.o_C4_8, &self.o_C4_18);
        self.o_C4_6 = self.c_C4_6.update(&self.o_C3_10, &self.o_C3_19, &self.o_C3_3, &self.o_C3_9);
        self.o_C3_15 = self.c_C3_15.update(&self.o_C2_18, &self.o_C2_13, &self.o_C2_6, &self.o_C2_15);
        self.o_C4_7 = self.c_C4_7.update(&self.o_C3_7, &self.o_C3_15, &self.o_C3_4, &self.o_C3_3);
        self.o_C5_7 = self.c_C5_7.update(&self.o_C4_9, &self.o_C4_6, &self.o_C4_7, &self.o_C4_11);
        self.o_C6_14 = self.c_C6_14.update(&self.o_C5_16, &self.o_C5_14, &self.o_C5_7, &self.o_C5_17);
        self.o_C4_1 = self.c_C4_1.update(&self.o_C3_12, &self.o_C3_17, &self.o_C3_11, &self.o_C3_15);
        self.o_C4_0 = self.c_C4_0.update(&self.o_C3_13, &self.o_C3_6, &self.o_C3_8, &self.o_C3_3);
        self.o_C2_11 = self.c_C2_11.update(&self.o_C1_17, &self.o_C1_18, &self.o_C1_5, &self.o_C1_2);
        self.o_C3_14 = self.c_C3_14.update(&self.o_C2_6, &self.o_C2_11, &self.o_C2_3, &self.o_C2_19);
        self.o_C4_17 = self.c_C4_17.update(&self.o_C3_3, &self.o_C3_14, &self.o_C3_5, &self.o_C3_7);
        self.o_C5_1 = self.c_C5_1.update(&self.o_C4_10, &self.o_C4_1, &self.o_C4_0, &self.o_C4_17);
        self.o_C4_19 = self.c_C4_19.update(&self.o_C3_17, &self.o_C3_9, &self.o_C3_19, &self.o_C3_8);
        self.o_C5_8 = self.c_C5_8.update(&self.o_C4_2, &self.o_C4_8, &self.o_C4_19, &self.o_C4_14);
        self.o_C5_2 = self.c_C5_2.update(&self.o_C4_9, &self.o_C4_10, &self.o_C4_14, &self.o_C4_12);
        self.o_C6_4 = self.c_C6_4.update(&self.o_C5_1, &self.o_C5_7, &self.o_C5_8, &self.o_C5_2);
        self.o_C5_10 = self.c_C5_10.update(&self.o_C4_12, &self.o_C4_9, &self.o_C4_1, &self.o_C4_10);
        self.o_C5_15 = self.c_C5_15.update(&self.o_C4_2, &self.o_C4_0, &self.o_C4_18, &self.o_C4_9);
        self.o_C6_15 = self.c_C6_15.update(&self.o_C5_10, &self.o_C5_15, &self.o_C5_18, &self.o_C5_7);
        self.o_C7_9 = self.c_C7_9.update(&self.o_C6_10, &self.o_C6_14, &self.o_C6_4, &self.o_C6_15);
        self.o_C5_13 = self.c_C5_13.update(&self.o_C4_7, &self.o_C4_19, &self.o_C4_0, &self.o_C4_18);
        self.o_C6_5 = self.c_C6_5.update(&self.o_C5_14, &self.o_C5_13, &self.o_C5_17, &self.o_C5_8);
        self.o_C6_17 = self.c_C6_17.update(&self.o_C5_7, &self.o_C5_1, &self.o_C5_2, &self.o_C5_16);
        self.o_C5_12 = self.c_C5_12.update(&self.o_C4_10, &self.o_C4_3, &self.o_C4_17, &self.o_C4_2);
        self.o_C6_11 = self.c_C6_11.update(&self.o_C5_8, &self.o_C5_19, &self.o_C5_12, &self.o_C5_17);
        self.o_C7_17 = self.c_C7_17.update(&self.o_C6_5, &self.o_C6_17, &self.o_C6_11, &self.o_C6_15);
        self.o_C5_5 = self.c_C5_5.update(&self.o_C4_6, &self.o_C4_17, &self.o_C4_15, &self.o_C4_11);
        self.o_C6_12 = self.c_C6_12.update(&self.o_C5_5, &self.o_C5_2, &self.o_C5_7, &self.o_C5_15);
        self.o_C5_9 = self.c_C5_9.update(&self.o_C4_2, &self.o_C4_18, &self.o_C4_10, &self.o_C4_7);
        self.o_C5_6 = self.c_C5_6.update(&self.o_C4_8, &self.o_C4_5, &self.o_C4_17, &self.o_C4_6);
        self.o_C6_0 = self.c_C6_0.update(&self.o_C5_16, &self.o_C5_9, &self.o_C5_4, &self.o_C5_6);
        self.o_C6_6 = self.c_C6_6.update(&self.o_C5_17, &self.o_C5_14, &self.o_C5_19, &self.o_C5_18);
        self.o_C7_12 = self.c_C7_12.update(&self.o_C6_12, &self.o_C6_17, &self.o_C6_0, &self.o_C6_6);
        self.o_C6_3 = self.c_C6_3.update(&self.o_C5_9, &self.o_C5_13, &self.o_C5_17, &self.o_C5_5);
        self.o_C5_0 = self.c_C5_0.update(&self.o_C4_15, &self.o_C4_10, &self.o_C4_3, &self.o_C4_6);
        self.o_C6_13 = self.c_C6_13.update(&self.o_C5_0, &self.o_C5_5, &self.o_C5_16, &self.o_C5_10);
        self.o_C7_10 = self.c_C7_10.update(&self.o_C6_6, &self.o_C6_3, &self.o_C6_13, &self.o_C6_17);
        self.o_C8_5 = self.c_C8_5.update(&self.o_C7_9, &self.o_C7_17, &self.o_C7_12, &self.o_C7_10);
        self.o_C5_11 = self.c_C5_11.update(&self.o_C4_5, &self.o_C4_10, &self.o_C4_9, &self.o_C4_7);
        self.o_C6_9 = self.c_C6_9.update(&self.o_C5_18, &self.o_C5_0, &self.o_C5_1, &self.o_C5_11);
        self.o_C7_19 = self.c_C7_19.update(&self.o_C6_6, &self.o_C6_9, &self.o_C6_3, &self.o_C6_0);
        self.o_C6_8 = self.c_C6_8.update(&self.o_C5_8, &self.o_C5_15, &self.o_C5_0, &self.o_C5_13);
        self.o_C6_19 = self.c_C6_19.update(&self.o_C5_9, &self.o_C5_19, &self.o_C5_18, &self.o_C5_11);
        self.o_C7_16 = self.c_C7_16.update(&self.o_C6_8, &self.o_C6_9, &self.o_C6_19, &self.o_C6_14);
        self.o_C6_16 = self.c_C6_16.update(&self.o_C5_13, &self.o_C5_10, &self.o_C5_17, &self.o_C5_8);
        self.o_C7_13 = self.c_C7_13.update(&self.o_C6_16, &self.o_C6_14, &self.o_C6_0, &self.o_C6_17);
        self.o_C8_9 = self.c_C8_9.update(&self.o_C7_19, &self.o_C7_16, &self.o_C7_13, &self.o_C7_9);
        self.o_C7_4 = self.c_C7_4.update(&self.o_C6_11, &self.o_C6_12, &self.o_C6_16, &self.o_C6_5);
        self.o_C8_17 = self.c_C8_17.update(&self.o_C7_13, &self.o_C7_17, &self.o_C7_9, &self.o_C7_4);
        self.o_C7_8 = self.c_C7_8.update(&self.o_C6_12, &self.o_C6_13, &self.o_C6_19, &self.o_C6_5);
        self.o_C6_2 = self.c_C6_2.update(&self.o_C5_19, &self.o_C5_17, &self.o_C5_6, &self.o_C5_5);
        self.o_C7_6 = self.c_C7_6.update(&self.o_C6_8, &self.o_C6_3, &self.o_C6_19, &self.o_C6_2);
        self.o_C6_7 = self.c_C6_7.update(&self.o_C5_0, &self.o_C5_12, &self.o_C5_10, &self.o_C5_5);
        self.o_C7_14 = self.c_C7_14.update(&self.o_C6_19, &self.o_C6_7, &self.o_C6_8, &self.o_C6_6);
        self.o_C8_8 = self.c_C8_8.update(&self.o_C7_8, &self.o_C7_16, &self.o_C7_6, &self.o_C7_14);
        self.o_C9_7 = self.c_C9_7.update(&self.o_C8_5, &self.o_C8_9, &self.o_C8_17, &self.o_C8_8);
        self.o_C7_0 = self.c_C7_0.update(&self.o_C6_5, &self.o_C6_14, &self.o_C6_2, &self.o_C6_3);
        self.o_C8_7 = self.c_C8_7.update(&self.o_C7_12, &self.o_C7_6, &self.o_C7_17, &self.o_C7_0);
        self.o_C7_18 = self.c_C7_18.update(&self.o_C6_13, &self.o_C6_3, &self.o_C6_6, &self.o_C6_12);
        self.o_C7_15 = self.c_C7_15.update(&self.o_C6_5, &self.o_C6_9, &self.o_C6_4, &self.o_C6_6);
        self.o_C6_18 = self.c_C6_18.update(&self.o_C5_11, &self.o_C5_5, &self.o_C5_16, &self.o_C5_6);
        self.o_C6_1 = self.c_C6_1.update(&self.o_C5_4, &self.o_C5_17, &self.o_C5_1, &self.o_C5_10);
        self.o_C7_3 = self.c_C7_3.update(&self.o_C6_18, &self.o_C6_1, &self.o_C6_15, &self.o_C6_12);
        self.o_C8_6 = self.c_C8_6.update(&self.o_C7_18, &self.o_C7_15, &self.o_C7_3, &self.o_C7_12);
        self.o_C8_2 = self.c_C8_2.update(&self.o_C7_18, &self.o_C7_9, &self.o_C7_13, &self.o_C7_16);
        self.o_C9_12 = self.c_C9_12.update(&self.o_C8_7, &self.o_C8_8, &self.o_C8_6, &self.o_C8_2);
        self.o_C7_5 = self.c_C7_5.update(&self.o_C6_17, &self.o_C6_1, &self.o_C6_16, &self.o_C6_2);
        self.o_C8_15 = self.c_C8_15.update(&self.o_C7_4, &self.o_C7_12, &self.o_C7_8, &self.o_C7_5);
        self.o_C7_11 = self.c_C7_11.update(&self.o_C6_3, &self.o_C6_9, &self.o_C6_8, &self.o_C6_7);
        self.o_C8_3 = self.c_C8_3.update(&self.o_C7_11, &self.o_C7_16, &self.o_C7_10, &self.o_C7_0);
        self.o_C7_2 = self.c_C7_2.update(&self.o_C6_4, &self.o_C6_8, &self.o_C6_13, &self.o_C6_6);
        self.o_C8_1 = self.c_C8_1.update(&self.o_C7_4, &self.o_C7_2, &self.o_C7_16, &self.o_C7_11);
        self.o_C9_19 = self.c_C9_19.update(&self.o_C8_15, &self.o_C8_3, &self.o_C8_5, &self.o_C8_1);
        self.o_C8_4 = self.c_C8_4.update(&self.o_C7_3, &self.o_C7_14, &self.o_C7_18, &self.o_C7_11);
        self.o_C8_18 = self.c_C8_18.update(&self.o_C7_14, &self.o_C7_8, &self.o_C7_15, &self.o_C7_5);
        self.o_C7_7 = self.c_C7_7.update(&self.o_C6_4, &self.o_C6_2, &self.o_C6_14, &self.o_C6_7);
        self.o_C8_14 = self.c_C8_14.update(&self.o_C7_7, &self.o_C7_9, &self.o_C7_0, &self.o_C7_13);
        self.o_C9_15 = self.c_C9_15.update(&self.o_C8_5, &self.o_C8_4, &self.o_C8_18, &self.o_C8_14);
        self.o_C10_3 = self.c_C10_3.update(&self.o_C9_7, &self.o_C9_12, &self.o_C9_19, &self.o_C9_15);
        self.o_C8_11 = self.c_C8_11.update(&self.o_C7_11, &self.o_C7_16, &self.o_C7_0, &self.o_C7_12);
        self.o_C8_16 = self.c_C8_16.update(&self.o_C7_2, &self.o_C7_0, &self.o_C7_11, &self.o_C7_8);
        self.o_C9_6 = self.c_C9_6.update(&self.o_C8_11, &self.o_C8_14, &self.o_C8_16, &self.o_C8_1);
        self.o_C9_5 = self.c_C9_5.update(&self.o_C8_8, &self.o_C8_2, &self.o_C8_18, &self.o_C8_16);
        self.o_C8_10 = self.c_C8_10.update(&self.o_C7_5, &self.o_C7_14, &self.o_C7_16, &self.o_C7_6);
        self.o_C9_4 = self.c_C9_4.update(&self.o_C8_16, &self.o_C8_6, &self.o_C8_7, &self.o_C8_10);
        self.o_C10_10 = self.c_C10_10.update(&self.o_C9_6, &self.o_C9_12, &self.o_C9_5, &self.o_C9_4);
        self.o_C8_12 = self.c_C8_12.update(&self.o_C7_18, &self.o_C7_13, &self.o_C7_12, &self.o_C7_10);
        self.o_C9_16 = self.c_C9_16.update(&self.o_C8_11, &self.o_C8_9, &self.o_C8_12, &self.o_C8_7);
        self.o_C7_1 = self.c_C7_1.update(&self.o_C6_19, &self.o_C6_16, &self.o_C6_12, &self.o_C6_5);
        self.o_C8_19 = self.c_C8_19.update(&self.o_C7_14, &self.o_C7_16, &self.o_C7_1, &self.o_C7_8);
        self.o_C9_14 = self.c_C9_14.update(&self.o_C8_19, &self.o_C8_8, &self.o_C8_14, &self.o_C8_16);
        self.o_C8_0 = self.c_C8_0.update(&self.o_C7_3, &self.o_C7_18, &self.o_C7_0, &self.o_C7_9);
        self.o_C9_10 = self.c_C9_10.update(&self.o_C8_7, &self.o_C8_8, &self.o_C8_19, &self.o_C8_0);
        self.o_C10_1 = self.c_C10_1.update(&self.o_C9_15, &self.o_C9_16, &self.o_C9_14, &self.o_C9_10);
        self.o_C10_4 = self.c_C10_4.update(&self.o_C9_14, &self.o_C9_12, &self.o_C9_5, &self.o_C9_7);
        self.o_C11_8 = self.c_C11_8.update(&self.o_C10_3, &self.o_C10_10, &self.o_C10_1, &self.o_C10_4);
        self.o_C9_2 = self.c_C9_2.update(&self.o_C8_5, &self.o_C8_16, &self.o_C8_19, &self.o_C8_2);
        self.o_C9_9 = self.c_C9_9.update(&self.o_C8_5, &self.o_C8_15, &self.o_C8_8, &self.o_C8_10);
        self.o_C10_13 = self.c_C10_13.update(&self.o_C9_4, &self.o_C9_2, &self.o_C9_14, &self.o_C9_9);
        self.o_C9_18 = self.c_C9_18.update(&self.o_C8_3, &self.o_C8_7, &self.o_C8_12, &self.o_C8_10);
        self.o_C9_3 = self.c_C9_3.update(&self.o_C8_12, &self.o_C8_8, &self.o_C8_9, &self.o_C8_6);
        self.o_C10_7 = self.c_C10_7.update(&self.o_C9_15, &self.o_C9_18, &self.o_C9_3, &self.o_C9_6);
        self.o_C9_13 = self.c_C9_13.update(&self.o_C8_5, &self.o_C8_18, &self.o_C8_14, &self.o_C8_4);
        self.o_C10_16 = self.c_C10_16.update(&self.o_C9_3, &self.o_C9_13, &self.o_C9_2, &self.o_C9_6);
        self.o_C11_15 = self.c_C11_15.update(&self.o_C10_3, &self.o_C10_13, &self.o_C10_7, &self.o_C10_16);
        self.o_C9_1 = self.c_C9_1.update(&self.o_C8_11, &self.o_C8_2, &self.o_C8_14, &self.o_C8_0);
        self.o_C8_13 = self.c_C8_13.update(&self.o_C7_19, &self.o_C7_18, &self.o_C7_2, &self.o_C7_15);
        self.o_C9_0 = self.c_C9_0.update(&self.o_C8_16, &self.o_C8_3, &self.o_C8_13, &self.o_C8_2);
        self.o_C9_17 = self.c_C9_17.update(&self.o_C8_3, &self.o_C8_6, &self.o_C8_9, &self.o_C8_2);
        self.o_C10_8 = self.c_C10_8.update(&self.o_C9_2, &self.o_C9_1, &self.o_C9_0, &self.o_C9_17);
        self.o_C9_8 = self.c_C9_8.update(&self.o_C8_11, &self.o_C8_7, &self.o_C8_12, &self.o_C8_17);
        self.o_C10_12 = self.c_C10_12.update(&self.o_C9_17, &self.o_C9_1, &self.o_C9_12, &self.o_C9_8);
        self.o_C10_9 = self.c_C10_9.update(&self.o_C9_15, &self.o_C9_10, &self.o_C9_12, &self.o_C9_9);
        self.o_C11_14 = self.c_C11_14.update(&self.o_C10_8, &self.o_C10_12, &self.o_C10_3, &self.o_C10_9);
        self.o_C10_2 = self.c_C10_2.update(&self.o_C9_8, &self.o_C9_3, &self.o_C9_5, &self.o_C9_18);
        self.o_C10_19 = self.c_C10_19.update(&self.o_C9_8, &self.o_C9_19, &self.o_C9_7, &self.o_C9_17);
        self.o_C11_11 = self.c_C11_11.update(&self.o_C10_8, &self.o_C10_2, &self.o_C10_19, &self.o_C10_10);
        self.o_C12_8 = self.c_C12_8.update(&self.o_C11_8, &self.o_C11_15, &self.o_C11_14, &self.o_C11_11);
        self.o_C10_17 = self.c_C10_17.update(&self.o_C9_0, &self.o_C9_15, &self.o_C9_4, &self.o_C9_8);
        self.o_C10_0 = self.c_C10_0.update(&self.o_C9_1, &self.o_C9_0, &self.o_C9_6, &self.o_C9_19);
        self.o_C10_5 = self.c_C10_5.update(&self.o_C9_7, &self.o_C9_9, &self.o_C9_14, &self.o_C9_12);
        self.o_C11_19 = self.c_C11_19.update(&self.o_C10_17, &self.o_C10_0, &self.o_C10_9, &self.o_C10_5);
        self.o_C10_14 = self.c_C10_14.update(&self.o_C9_0, &self.o_C9_1, &self.o_C9_17, &self.o_C9_18);
        self.o_C11_10 = self.c_C11_10.update(&self.o_C10_12, &self.o_C10_14, &self.o_C10_0, &self.o_C10_16);
        self.o_C10_15 = self.c_C10_15.update(&self.o_C9_16, &self.o_C9_4, &self.o_C9_1, &self.o_C9_8);
        self.o_C11_17 = self.c_C11_17.update(&self.o_C10_16, &self.o_C10_12, &self.o_C10_15, &self.o_C10_3);
        self.o_C10_6 = self.c_C10_6.update(&self.o_C9_6, &self.o_C9_14, &self.o_C9_8, &self.o_C9_10);
        self.o_C11_16 = self.c_C11_16.update(&self.o_C10_17, &self.o_C10_6, &self.o_C10_10, &self.o_C10_19);
        self.o_C12_13 = self.c_C12_13.update(&self.o_C11_19, &self.o_C11_10, &self.o_C11_17, &self.o_C11_16);
        self.o_C11_3 = self.c_C11_3.update(&self.o_C10_17, &self.o_C10_13, &self.o_C10_2, &self.o_C10_8);
        self.o_C11_4 = self.c_C11_4.update(&self.o_C10_19, &self.o_C10_2, &self.o_C10_8, &self.o_C10_5);
        self.o_C12_6 = self.c_C12_6.update(&self.o_C11_16, &self.o_C11_3, &self.o_C11_4, &self.o_C11_10);
        self.o_C10_18 = self.c_C10_18.update(&self.o_C9_6, &self.o_C9_14, &self.o_C9_12, &self.o_C9_10);
        self.o_C11_6 = self.c_C11_6.update(&self.o_C10_13, &self.o_C10_1, &self.o_C10_18, &self.o_C10_2);
        self.o_C11_12 = self.c_C11_12.update(&self.o_C10_2, &self.o_C10_9, &self.o_C10_1, &self.o_C10_12);
        self.o_C12_0 = self.c_C12_0.update(&self.o_C11_6, &self.o_C11_11, &self.o_C11_12, &self.o_C11_16);
        self.o_C13_4 = self.c_C13_4.update(&self.o_C12_8, &self.o_C12_13, &self.o_C12_6, &self.o_C12_0);
        self.o_C10_11 = self.c_C10_11.update(&self.o_C9_0, &self.o_C9_19, &self.o_C9_12, &self.o_C9_4);
        self.o_C11_2 = self.c_C11_2.update(&self.o_C10_11, &self.o_C10_17, &self.o_C10_13, &self.o_C10_6);
        self.o_C11_18 = self.c_C11_18.update(&self.o_C10_4, &self.o_C10_14, &self.o_C10_16, &self.o_C10_17);
        self.o_C12_18 = self.c_C12_18.update(&self.o_C11_8, &self.o_C11_2, &self.o_C11_16, &self.o_C11_18);
        self.o_C11_9 = self.c_C11_9.update(&self.o_C10_17, &self.o_C10_1, &self.o_C10_14, &self.o_C10_4);
        self.o_C12_4 = self.c_C12_4.update(&self.o_C11_9, &self.o_C11_10, &self.o_C11_11, &self.o_C11_8);
        self.o_C12_17 = self.c_C12_17.update(&self.o_C11_17, &self.o_C11_8, &self.o_C11_3, &self.o_C11_6);
        self.o_C11_0 = self.c_C11_0.update(&self.o_C10_1, &self.o_C10_18, &self.o_C10_5, &self.o_C10_11);
        self.o_C12_5 = self.c_C12_5.update(&self.o_C11_10, &self.o_C11_16, &self.o_C11_18, &self.o_C11_0);
        self.o_C13_13 = self.c_C13_13.update(&self.o_C12_18, &self.o_C12_4, &self.o_C12_17, &self.o_C12_5);
        self.o_C11_5 = self.c_C11_5.update(&self.o_C10_3, &self.o_C10_4, &self.o_C10_1, &self.o_C10_6);
        self.o_C12_14 = self.c_C12_14.update(&self.o_C11_5, &self.o_C11_0, &self.o_C11_4, &self.o_C11_8);
        self.o_C11_13 = self.c_C11_13.update(&self.o_C10_1, &self.o_C10_8, &self.o_C10_10, &self.o_C10_4);
        self.o_C12_3 = self.c_C12_3.update(&self.o_C11_9, &self.o_C11_17, &self.o_C11_10, &self.o_C11_13);
        self.o_C11_7 = self.c_C11_7.update(&self.o_C10_16, &self.o_C10_15, &self.o_C10_19, &self.o_C10_11);
        self.o_C12_15 = self.c_C12_15.update(&self.o_C11_7, &self.o_C11_18, &self.o_C11_4, &self.o_C11_3);
        self.o_C12_11 = self.c_C12_11.update(&self.o_C11_7, &self.o_C11_18, &self.o_C11_10, &self.o_C11_11);
        self.o_C13_19 = self.c_C13_19.update(&self.o_C12_14, &self.o_C12_3, &self.o_C12_15, &self.o_C12_11);
        self.o_C13_0 = self.c_C13_0.update(&self.o_C12_13, &self.o_C12_0, &self.o_C12_11, &self.o_C12_15);
        self.o_C14_0 = self.c_C14_0.update(&self.o_C13_4, &self.o_C13_13, &self.o_C13_19, &self.o_C13_0);
        self.o_C12_12 = self.c_C12_12.update(&self.o_C11_11, &self.o_C11_12, &self.o_C11_9, &self.o_C11_14);
        self.o_C13_17 = self.c_C13_17.update(&self.o_C12_12, &self.o_C12_8, &self.o_C12_14, &self.o_C12_18);
        self.o_C11_1 = self.c_C11_1.update(&self.o_C10_13, &self.o_C10_17, &self.o_C10_16, &self.o_C10_1);
        self.o_C12_9 = self.c_C12_9.update(&self.o_C11_12, &self.o_C11_2, &self.o_C11_1, &self.o_C11_4);
        self.o_C12_16 = self.c_C12_16.update(&self.o_C11_5, &self.o_C11_13, &self.o_C11_1, &self.o_C11_3);
        self.o_C12_10 = self.c_C12_10.update(&self.o_C11_1, &self.o_C11_16, &self.o_C11_15, &self.o_C11_8);
        self.o_C12_19 = self.c_C12_19.update(&self.o_C11_2, &self.o_C11_6, &self.o_C11_5, &self.o_C11_16);
        self.o_C13_9 = self.c_C13_9.update(&self.o_C12_9, &self.o_C12_16, &self.o_C12_10, &self.o_C12_19);
        self.o_C13_15 = self.c_C13_15.update(&self.o_C12_11, &self.o_C12_13, &self.o_C12_12, &self.o_C12_9);
        self.o_C12_2 = self.c_C12_2.update(&self.o_C11_4, &self.o_C11_18, &self.o_C11_2, &self.o_C11_1);
        self.o_C13_6 = self.c_C13_6.update(&self.o_C12_2, &self.o_C12_12, &self.o_C12_16, &self.o_C12_13);
        self.o_C14_19 = self.c_C14_19.update(&self.o_C13_17, &self.o_C13_9, &self.o_C13_15, &self.o_C13_6);
        self.o_C13_5 = self.c_C13_5.update(&self.o_C12_17, &self.o_C12_12, &self.o_C12_16, &self.o_C12_15);
        self.o_C12_7 = self.c_C12_7.update(&self.o_C11_10, &self.o_C11_19, &self.o_C11_2, &self.o_C11_14);
        self.o_C13_2 = self.c_C13_2.update(&self.o_C12_7, &self.o_C12_13, &self.o_C12_14, &self.o_C12_11);
        self.o_C14_9 = self.c_C14_9.update(&self.o_C13_5, &self.o_C13_0, &self.o_C13_2, &self.o_C13_13);
        self.o_C12_1 = self.c_C12_1.update(&self.o_C11_10, &self.o_C11_3, &self.o_C11_13, &self.o_C11_11);
        self.o_C13_7 = self.c_C13_7.update(&self.o_C12_1, &self.o_C12_11, &self.o_C12_14, &self.o_C12_0);
        self.o_C13_3 = self.c_C13_3.update(&self.o_C12_17, &self.o_C12_6, &self.o_C12_15, &self.o_C12_2);
        self.o_C13_8 = self.c_C13_8.update(&self.o_C12_6, &self.o_C12_9, &self.o_C12_0, &self.o_C12_3);
        self.o_C14_7 = self.c_C14_7.update(&self.o_C13_7, &self.o_C13_0, &self.o_C13_3, &self.o_C13_8);
        self.o_C15_18 = self.c_C15_18.update(&self.o_C14_0, &self.o_C14_19, &self.o_C14_9, &self.o_C14_7);
        self.o_C13_1 = self.c_C13_1.update(&self.o_C12_9, &self.o_C12_7, &self.o_C12_6, &self.o_C12_15);
        self.o_C14_10 = self.c_C14_10.update(&self.o_C13_19, &self.o_C13_1, &self.o_C13_17, &self.o_C13_6);
        self.o_C13_14 = self.c_C13_14.update(&self.o_C12_8, &self.o_C12_0, &self.o_C12_13, &self.o_C12_1);
        self.o_C13_16 = self.c_C13_16.update(&self.o_C12_0, &self.o_C12_2, &self.o_C12_18, &self.o_C12_19);
        self.o_C14_15 = self.c_C14_15.update(&self.o_C13_3, &self.o_C13_14, &self.o_C13_9, &self.o_C13_16);
        self.o_C14_3 = self.c_C14_3.update(&self.o_C13_16, &self.o_C13_9, &self.o_C13_13, &self.o_C13_8);
        self.o_C15_0 = self.c_C15_0.update(&self.o_C14_19, &self.o_C14_10, &self.o_C14_15, &self.o_C14_3);
        self.o_C13_11 = self.c_C13_11.update(&self.o_C12_13, &self.o_C12_18, &self.o_C12_9, &self.o_C12_14);
        self.o_C14_11 = self.c_C14_11.update(&self.o_C13_17, &self.o_C13_13, &self.o_C13_11, &self.o_C13_1);
        self.o_C15_9 = self.c_C15_9.update(&self.o_C14_11, &self.o_C14_15, &self.o_C14_7, &self.o_C14_10);
        self.o_C13_10 = self.c_C13_10.update(&self.o_C12_16, &self.o_C12_13, &self.o_C12_17, &self.o_C12_19);
        self.o_C14_4 = self.c_C14_4.update(&self.o_C13_13, &self.o_C13_10, &self.o_C13_15, &self.o_C13_6);
        self.o_C13_12 = self.c_C13_12.update(&self.o_C12_9, &self.o_C12_4, &self.o_C12_16, &self.o_C12_14);
        self.o_C14_8 = self.c_C14_8.update(&self.o_C13_4, &self.o_C13_15, &self.o_C13_3, &self.o_C13_12);
        self.o_C13_18 = self.c_C13_18.update(&self.o_C12_11, &self.o_C12_15, &self.o_C12_10, &self.o_C12_12);
        self.o_C14_2 = self.c_C14_2.update(&self.o_C13_18, &self.o_C13_9, &self.o_C13_13, &self.o_C13_8);
        self.o_C15_14 = self.c_C15_14.update(&self.o_C14_4, &self.o_C14_8, &self.o_C14_7, &self.o_C14_2);
        self.o_C16_19 = self.c_C16_19.update(&self.o_C15_18, &self.o_C15_0, &self.o_C15_9, &self.o_C15_14);
        self.o_C14_1 = self.c_C14_1.update(&self.o_C13_5, &self.o_C13_8, &self.o_C13_11, &self.o_C13_4);
        self.o_C15_1 = self.c_C15_1.update(&self.o_C14_0, &self.o_C14_11, &self.o_C14_8, &self.o_C14_1);
        self.o_C14_18 = self.c_C14_18.update(&self.o_C13_5, &self.o_C13_16, &self.o_C13_8, &self.o_C13_13);
        self.o_C14_6 = self.c_C14_6.update(&self.o_C13_2, &self.o_C13_4, &self.o_C13_6, &self.o_C13_18);
        self.o_C14_16 = self.c_C14_16.update(&self.o_C13_15, &self.o_C13_12, &self.o_C13_3, &self.o_C13_19);
        self.o_C15_16 = self.c_C15_16.update(&self.o_C14_7, &self.o_C14_18, &self.o_C14_6, &self.o_C14_16);
        self.o_C14_17 = self.c_C14_17.update(&self.o_C13_3, &self.o_C13_4, &self.o_C13_12, &self.o_C13_6);
        self.o_C15_3 = self.c_C15_3.update(&self.o_C14_7, &self.o_C14_16, &self.o_C14_8, &self.o_C14_17);
        self.o_C14_13 = self.c_C14_13.update(&self.o_C13_8, &self.o_C13_19, &self.o_C13_5, &self.o_C13_15);
        self.o_C15_5 = self.c_C15_5.update(&self.o_C14_8, &self.o_C14_6, &self.o_C14_13, &self.o_C14_1);
        self.o_C16_13 = self.c_C16_13.update(&self.o_C15_1, &self.o_C15_16, &self.o_C15_3, &self.o_C15_5);
        self.o_C15_17 = self.c_C15_17.update(&self.o_C14_18, &self.o_C14_9, &self.o_C14_13, &self.o_C14_10);
        self.o_C15_11 = self.c_C15_11.update(&self.o_C14_1, &self.o_C14_16, &self.o_C14_10, &self.o_C14_18);
        self.o_C15_19 = self.c_C15_19.update(&self.o_C14_2, &self.o_C14_7, &self.o_C14_8, &self.o_C14_10);
        self.o_C16_18 = self.c_C16_18.update(&self.o_C15_17, &self.o_C15_11, &self.o_C15_19, &self.o_C15_16);
        self.o_C15_7 = self.c_C15_7.update(&self.o_C14_8, &self.o_C14_19, &self.o_C14_15, &self.o_C14_9);
        self.o_C15_4 = self.c_C15_4.update(&self.o_C14_7, &self.o_C14_13, &self.o_C14_4, &self.o_C14_17);
        self.o_C16_12 = self.c_C16_12.update(&self.o_C15_9, &self.o_C15_7, &self.o_C15_19, &self.o_C15_4);
        self.o_C17_16 = self.c_C17_16.update(&self.o_C16_19, &self.o_C16_13, &self.o_C16_18, &self.o_C16_12);
        self.o_C15_6 = self.c_C15_6.update(&self.o_C14_17, &self.o_C14_16, &self.o_C14_4, &self.o_C14_13);
        self.o_C15_13 = self.c_C15_13.update(&self.o_C14_15, &self.o_C14_10, &self.o_C14_3, &self.o_C14_4);
        self.o_C15_8 = self.c_C15_8.update(&self.o_C14_8, &self.o_C14_15, &self.o_C14_6, &self.o_C14_18);
        self.o_C16_14 = self.c_C16_14.update(&self.o_C15_7, &self.o_C15_6, &self.o_C15_13, &self.o_C15_8);
        self.o_C15_12 = self.c_C15_12.update(&self.o_C14_4, &self.o_C14_6, &self.o_C14_10, &self.o_C14_15);
        self.o_C16_0 = self.c_C16_0.update(&self.o_C15_8, &self.o_C15_16, &self.o_C15_12, &self.o_C15_0);
        self.o_C14_14 = self.c_C14_14.update(&self.o_C13_1, &self.o_C13_6, &self.o_C13_2, &self.o_C13_12);
        self.o_C15_2 = self.c_C15_2.update(&self.o_C14_17, &self.o_C14_14, &self.o_C14_9, &self.o_C14_3);
        self.o_C16_17 = self.c_C16_17.update(&self.o_C15_12, &self.o_C15_3, &self.o_C15_11, &self.o_C15_2);
        self.o_C17_6 = self.c_C17_6.update(&self.o_C16_14, &self.o_C16_0, &self.o_C16_13, &self.o_C16_17);
        self.o_C14_5 = self.c_C14_5.update(&self.o_C13_15, &self.o_C13_12, &self.o_C13_13, &self.o_C13_2);
        self.o_C15_15 = self.c_C15_15.update(&self.o_C14_17, &self.o_C14_1, &self.o_C14_5, &self.o_C14_3);
        self.o_C16_10 = self.c_C16_10.update(&self.o_C15_2, &self.o_C15_18, &self.o_C15_16, &self.o_C15_15);
        self.o_C16_3 = self.c_C16_3.update(&self.o_C15_11, &self.o_C15_2, &self.o_C15_18, &self.o_C15_3);
        self.o_C16_16 = self.c_C16_16.update(&self.o_C15_16, &self.o_C15_8, &self.o_C15_15, &self.o_C15_4);
        self.o_C16_5 = self.c_C16_5.update(&self.o_C15_16, &self.o_C15_1, &self.o_C15_11, &self.o_C15_0);
        self.o_C17_19 = self.c_C17_19.update(&self.o_C16_10, &self.o_C16_3, &self.o_C16_16, &self.o_C16_5);
        self.o_C16_9 = self.c_C16_9.update(&self.o_C15_11, &self.o_C15_4, &self.o_C15_8, &self.o_C15_12);
        self.o_C15_10 = self.c_C15_10.update(&self.o_C14_5, &self.o_C14_19, &self.o_C14_14, &self.o_C14_4);
        self.o_C16_4 = self.c_C16_4.update(&self.o_C15_9, &self.o_C15_10, &self.o_C15_7, &self.o_C15_8);
        self.o_C17_2 = self.c_C17_2.update(&self.o_C16_9, &self.o_C16_5, &self.o_C16_4, &self.o_C16_12);
        self.o_C18_18 = self.c_C18_18.update(&self.o_C17_16, &self.o_C17_6, &self.o_C17_19, &self.o_C17_2);
        self.o_C16_7 = self.c_C16_7.update(&self.o_C15_7, &self.o_C15_3, &self.o_C15_10, &self.o_C15_8);
        self.o_C16_1 = self.c_C16_1.update(&self.o_C15_3, &self.o_C15_10, &self.o_C15_11, &self.o_C15_4);
        self.o_C17_18 = self.c_C17_18.update(&self.o_C16_7, &self.o_C16_16, &self.o_C16_4, &self.o_C16_1);
        self.o_C16_2 = self.c_C16_2.update(&self.o_C15_3, &self.o_C15_8, &self.o_C15_4, &self.o_C15_1);
        self.o_C17_0 = self.c_C17_0.update(&self.o_C16_4, &self.o_C16_19, &self.o_C16_2, &self.o_C16_18);
        self.o_C16_11 = self.c_C16_11.update(&self.o_C15_18, &self.o_C15_13, &self.o_C15_17, &self.o_C15_12);
        self.o_C16_8 = self.c_C16_8.update(&self.o_C15_0, &self.o_C15_16, &self.o_C15_10, &self.o_C15_3);
        self.o_C17_15 = self.c_C17_15.update(&self.o_C16_13, &self.o_C16_11, &self.o_C16_8, &self.o_C16_1);
        self.o_C18_1 = self.c_C18_1.update(&self.o_C17_18, &self.o_C17_0, &self.o_C17_15, &self.o_C17_6);
        self.o_C17_14 = self.c_C17_14.update(&self.o_C16_5, &self.o_C16_3, &self.o_C16_1, &self.o_C16_10);
        self.o_C16_15 = self.c_C16_15.update(&self.o_C15_17, &self.o_C15_0, &self.o_C15_8, &self.o_C15_19);
        self.o_C17_12 = self.c_C17_12.update(&self.o_C16_18, &self.o_C16_11, &self.o_C16_15, &self.o_C16_7);
        self.o_C17_7 = self.c_C17_7.update(&self.o_C16_8, &self.o_C16_11, &self.o_C16_13, &self.o_C16_12);
        self.o_C18_12 = self.c_C18_12.update(&self.o_C17_14, &self.o_C17_12, &self.o_C17_7, &self.o_C17_18);
        self.o_C17_5 = self.c_C17_5.update(&self.o_C16_4, &self.o_C16_12, &self.o_C16_17, &self.o_C16_3);
        self.o_C18_2 = self.c_C18_2.update(&self.o_C17_12, &self.o_C17_5, &self.o_C17_19, &self.o_C17_7);
        self.o_C19_0 = self.c_C19_0.update(&self.o_C18_18, &self.o_C18_1, &self.o_C18_12, &self.o_C18_2);
        self.o_C17_13 = self.c_C17_13.update(&self.o_C16_19, &self.o_C16_7, &self.o_C16_3, &self.o_C16_11);
        self.o_C16_6 = self.c_C16_6.update(&self.o_C15_2, &self.o_C15_4, &self.o_C15_12, &self.o_C15_11);
        self.o_C17_1 = self.c_C17_1.update(&self.o_C16_6, &self.o_C16_15, &self.o_C16_10, &self.o_C16_11);
        self.o_C18_17 = self.c_C18_17.update(&self.o_C17_6, &self.o_C17_13, &self.o_C17_7, &self.o_C17_1);
        self.o_C17_3 = self.c_C17_3.update(&self.o_C16_14, &self.o_C16_12, &self.o_C16_3, &self.o_C16_4);
        self.o_C17_10 = self.c_C17_10.update(&self.o_C16_3, &self.o_C16_18, &self.o_C16_4, &self.o_C16_16);
        self.o_C17_17 = self.c_C17_17.update(&self.o_C16_11, &self.o_C16_9, &self.o_C16_10, &self.o_C16_14);
        self.o_C18_3 = self.c_C18_3.update(&self.o_C17_3, &self.o_C17_7, &self.o_C17_10, &self.o_C17_17);
        self.o_C18_15 = self.c_C18_15.update(&self.o_C17_2, &self.o_C17_17, &self.o_C17_18, &self.o_C17_5);
        self.o_C19_1 = self.c_C19_1.update(&self.o_C18_17, &self.o_C18_3, &self.o_C18_15, &self.o_C18_1);
        self.o_C18_16 = self.c_C18_16.update(&self.o_C17_5, &self.o_C17_12, &self.o_C17_0, &self.o_C17_16);
        self.o_C17_8 = self.c_C17_8.update(&self.o_C16_19, &self.o_C16_14, &self.o_C16_1, &self.o_C16_3);
        self.o_C17_4 = self.c_C17_4.update(&self.o_C16_8, &self.o_C16_9, &self.o_C16_0, &self.o_C16_17);
        self.o_C18_7 = self.c_C18_7.update(&self.o_C17_8, &self.o_C17_4, &self.o_C17_18, &self.o_C17_0);
        self.o_C18_0 = self.c_C18_0.update(&self.o_C17_17, &self.o_C17_15, &self.o_C17_10, &self.o_C17_3);
        self.o_C19_2 = self.c_C19_2.update(&self.o_C18_16, &self.o_C18_7, &self.o_C18_0, &self.o_C18_17);
        self.o_C18_9 = self.c_C18_9.update(&self.o_C17_2, &self.o_C17_5, &self.o_C17_14, &self.o_C17_12);
        self.o_C18_14 = self.c_C18_14.update(&self.o_C17_5, &self.o_C17_19, &self.o_C17_10, &self.o_C17_7);
        self.o_C18_8 = self.c_C18_8.update(&self.o_C17_12, &self.o_C17_13, &self.o_C17_3, &self.o_C17_0);
        self.o_C18_13 = self.c_C18_13.update(&self.o_C17_0, &self.o_C17_17, &self.o_C17_7, &self.o_C17_13);
        self.o_C19_3 = self.c_C19_3.update(&self.o_C18_9, &self.o_C18_14, &self.o_C18_8, &self.o_C18_13);
        self.o_C18_5 = self.c_C18_5.update(&self.o_C17_15, &self.o_C17_6, &self.o_C17_13, &self.o_C17_14);
        self.o_C17_11 = self.c_C17_11.update(&self.o_C16_16, &self.o_C16_11, &self.o_C16_17, &self.o_C16_8);
        self.o_C18_4 = self.c_C18_4.update(&self.o_C17_7, &self.o_C17_14, &self.o_C17_15, &self.o_C17_11);
        self.o_C17_9 = self.c_C17_9.update(&self.o_C16_15, &self.o_C16_1, &self.o_C16_0, &self.o_C16_18);
        self.o_C18_10 = self.c_C18_10.update(&self.o_C17_16, &self.o_C17_9, &self.o_C17_4, &self.o_C17_17);
        self.o_C19_4 = self.c_C19_4.update(&self.o_C18_5, &self.o_C18_4, &self.o_C18_17, &self.o_C18_10);
        self.o_C19_5 = self.c_C19_5.update(&self.o_C18_17, &self.o_C18_14, &self.o_C18_16, &self.o_C18_13);
        self.o_C18_19 = self.c_C18_19.update(&self.o_C17_7, &self.o_C17_12, &self.o_C17_14, &self.o_C17_3);
        self.o_C19_6 = self.c_C19_6.update(&self.o_C18_17, &self.o_C18_5, &self.o_C18_12, &self.o_C18_19);
        self.o_C18_6 = self.c_C18_6.update(&self.o_C17_12, &self.o_C17_17, &self.o_C17_3, &self.o_C17_15);
        self.o_C18_11 = self.c_C18_11.update(&self.o_C17_16, &self.o_C17_3, &self.o_C17_8, &self.o_C17_0);
        self.o_C19_7 = self.c_C19_7.update(&self.o_C18_6, &self.o_C18_15, &self.o_C18_8, &self.o_C18_11);
        self.o_C19_8 = self.c_C19_8.update(&self.o_C18_4, &self.o_C18_8, &self.o_C18_18, &self.o_C18_5);
        self.o_C19_9 = self.c_C19_9.update(&self.o_C18_19, &self.o_C18_2, &self.o_C18_11, &self.o_C18_10);
        self.o_C19_10 = self.c_C19_10.update(&self.o_C18_4, &self.o_C18_8, &self.o_C18_18, &self.o_C18_17);
        self.o_C19_11 = self.c_C19_11.update(&self.o_C18_11, &self.o_C18_12, &self.o_C18_8, &self.o_C18_14);
        self.o_C19_12 = self.c_C19_12.update(&self.o_C18_0, &self.o_C18_4, &self.o_C18_18, &self.o_C18_8);
        self.o_C19_13 = self.c_C19_13.update(&self.o_C18_7, &self.o_C18_6, &self.o_C18_2, &self.o_C18_18);
        self.o_C19_14 = self.c_C19_14.update(&self.o_C18_17, &self.o_C18_13, &self.o_C18_7, &self.o_C18_4);
        self.o_C19_15 = self.c_C19_15.update(&self.o_C18_17, &self.o_C18_14, &self.o_C18_12, &self.o_C18_6);
        self.o_C19_16 = self.c_C19_16.update(&self.o_C18_2, &self.o_C18_19, &self.o_C18_4, &self.o_C18_1);
        self.o_C19_17 = self.c_C19_17.update(&self.o_C18_0, &self.o_C18_12, &self.o_C18_18, &self.o_C18_13);
        self.o_C19_18 = self.c_C19_18.update(&self.o_C18_4, &self.o_C18_18, &self.o_C18_19, &self.o_C18_2);
        self.o_C19_19 = self.c_C19_19.update(&self.o_C18_7, &self.o_C18_12, &self.o_C18_4, &self.o_C18_9);
        0 ^ self.o_C19_0 ^ self.o_C19_1 ^ self.o_C19_2 ^ self.o_C19_3 ^ self.o_C19_4 ^ self.o_C19_5 ^ self.o_C19_6 ^ self.o_C19_7 ^ self.o_C19_8 ^ self.o_C19_9 ^ self.o_C19_10 ^ self.o_C19_11 ^ self.o_C19_12 ^ self.o_C19_13 ^ self.o_C19_14 ^ self.o_C19_15 ^ self.o_C19_16 ^ self.o_C19_17 ^ self.o_C19_18 ^ self.o_C19_19
    }
}
const WARMUP: u32 = 5_000; // untimed runs per side
const RUNS: u32 = 50_000; // timed runs per side per round
const ROUNDS: usize = 11; // timed rounds; the side that runs first alternates
fn stats(v: &mut Vec<f64>) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[0], v[v.len() / 2])
}
#[inline(never)]
fn run_flat(p: &mut dataflow_flat::Pipeline<flat::Poll, (flat::App,)>) -> u64 { p.push(flat::Poll); flat::OUT.load(std::sync::atomic::Ordering::Relaxed) }
fn main() {
    let mut b = dataflow_flat::Pipeline::<flat::Poll, (flat::App,)>::build(dataflow_flat::Configs::new()).unwrap();
    let mut xb = 0;
    let mut tf: Vec<f64> = Vec::new();
    for _ in 0..WARMUP { xb = run_flat(&mut b); }
    let mut h = Hand::new();
    let mut xh = 0;
    let mut th: Vec<f64> = Vec::new();
    for _ in 0..WARMUP { xh = h.run(); }
    for r in 0..ROUNDS { if r % 2 == 0 { { let t = Instant::now(); for _ in 0..RUNS { xb = run_flat(&mut b); } let el = t.elapsed(); black_box(xb); tf.push(el.as_nanos() as f64 / RUNS as f64); } { let t = Instant::now(); for _ in 0..RUNS { xh = h.run(); } let el = t.elapsed(); black_box(xh); th.push(el.as_nanos() as f64 / RUNS as f64); } } else { { let t = Instant::now(); for _ in 0..RUNS { xh = h.run(); } let el = t.elapsed(); black_box(xh); th.push(el.as_nanos() as f64 / RUNS as f64); } { let t = Instant::now(); for _ in 0..RUNS { xb = run_flat(&mut b); } let el = t.elapsed(); black_box(xb); tf.push(el.as_nanos() as f64 / RUNS as f64); } } }
    let (mn, md) = stats(&mut tf);
    println!("flat:         min {:>6.1}  median {:>6.1} ns/run  ({} components, {} rounds x {} runs)", mn, md, b.len(), ROUNDS, RUNS);
    let (mn, md) = stats(&mut th);
    println!("hand-written: min {:>6.1}  median {:>6.1} ns/run", mn, md);
    assert_eq!(xb, xh); println!("results agree");
}