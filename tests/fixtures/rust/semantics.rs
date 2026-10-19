//! Exercises Rust naming, documentation and cognitive complexity rules.

use std::fmt;

/// Adds the primes up to `max` (SonarSource 1.7 scores it 7).
fn sum_of_primes(max: u32) -> u32 {
    let mut total = 0;
    'out: for i in 1..=max {
        for j in 2..i {
            if i % j == 0 {
                continue 'out;
            }
        }
        total += i;
    }
    total
}

fn get_words(number: u32) -> &'static str {
    match number {
        1 => "one",
        2 => "a couple",
        3 => "a few",
        _ => "lots",
    }
}

fn mixed(a: bool, b: bool, c: bool, d: Option<u32>) -> Option<u32> {
    let x = d?;
    if a && b || c {
        return Some(x);
    } else if a {
        return None;
    } else {
        let f = |y: u32| if y > 1 { y } else { 0 };
        Some(f(x))
    }
}

pub struct Shape;

impl Shape {
    /// Makes a shape.
    #[inline]
    pub fn new() -> Self {
        fn helper() -> u32 {
            1
        }
        helper();
        Shape
    }
}

trait Area {
    fn area(&self) -> u32;
    fn double(&self) -> u32 {
        self.area() * 2
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shape")
    }
}

fn new() -> Shape {
    Shape
}

fn usage() -> &'static str {
    r#"
usage: topo
"#
}

impl Area for &'static Shape {
    fn area(&self) -> u32 {
        1
    }
}

/// Counts down while a value remains.
#[allow(dead_code)] // kept for the example
fn count_down(mut a: Option<u32>, b: bool, c: bool) -> u32 {
    while let Some(x) = a && b {
        a = x.checked_sub(1);
    }
    if let Some(x) = a && b && (c || b) { x } else { 0 }
}

//// A plain comment, not documentation.
fn banner() {}
