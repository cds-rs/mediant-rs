//! # Farey Approximation
//!
//! Approximates real numbers as fractions using the Farey sequence properties.
//!
//! The Farey sequence F_n is the sequence of completely reduced fractions between
//! 0 and 1, with denominators ≤ n, arranged in increasing order. A key property
//! is that for any two adjacent fractions a/b and c/d in a Farey sequence, their
//! mediant (a+c)/(b+d) lies between them.
//!
//! This tool uses the mediant property to perform a binary search, narrowing
//! bounds until finding the closest rational approximation to any real number.

use bpaf::Bpaf;
use std::fmt;

#[derive(Clone, Debug, Bpaf)]
#[bpaf(options, version)]
struct Args {
    /// The real number
    number: f64,
}

/// A fraction represented as numerator/denominator.
///
/// Fractions are the building blocks of Farey sequences. In the context of this
/// algorithm, we maintain two fractions (left and right bounds) and repeatedly
/// compute their mediant to converge on a target value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fraction {
    numerator: u64,
    denominator: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DivByZeroError;

impl fmt::Display for DivByZeroError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "division by zero: denominator cannot be zero")
    }
}

impl std::error::Error for DivByZeroError {}

impl Fraction {
    fn new(numerator: u64, denominator: u64) -> Result<Self, DivByZeroError> {
        if denominator == 0 {
            Err(DivByZeroError)
        } else {
            Ok(Self { numerator, denominator })
        }
    }

    /// Returns the decimal value of this fraction.
    fn value(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// Computes the mediant of two fractions.
    ///
    /// The mediant of a/b and c/d is (a+c)/(b+d). This is NOT the arithmetic mean,
    /// but rather "Farey addition". The mediant has a key property: if a/b < c/d,
    /// then a/b < mediant < c/d. This property enables binary search over rationals.
    ///
    /// Example: mediant of 1/2 and 1/3 is (1+1)/(2+3) = 2/5
    fn mediant(&self, other: &Fraction) -> Result<Self, DivByZeroError> {
        Self::new(
            self.numerator + other.numerator,
            self.denominator + other.denominator,
        )
    }
}

/// Format the fraction as follows:
///               27450985
/// 0.33333339 ≈ ----------
///               82352941
impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value_str = format!("{:.15}", self.value())
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        let frac_width = self.numerator.max(self.denominator).to_string().len();
        let sep_width = frac_width + 2;
        // Right-align fractions to end at the same column as the separator
        let pad = value_str.len() + 3 + sep_width;

        write!(
            f,
            "\n{:>pad$}\n{value_str} ≈ {:-<sep_width$}\n{:>pad$}\n\n$ {value_str} ≈ frac({},{}) $",
            self.numerator, "", self.denominator, self.numerator, self.denominator
        )
    }
}

/// Approximates a real number as a fraction using the Farey/mediant algorithm.
///
/// # Algorithm
///
/// 1. Start with two bounds: left = floor(x)/1 and right = ceil(x)/1
/// 2. Compute the mediant of left and right
/// 3. If mediant equals target (within epsilon), we're done
/// 4. If mediant > target, it becomes the new right bound (search left half)
/// 5. If mediant < target, it becomes the new left bound (search right half)
/// 6. Repeat until convergence
///
/// This is essentially binary search over the Stern-Brocot tree, which contains
/// all positive rationals exactly once. The mediant operation naturally traverses
/// this tree, guaranteeing we find the best rational approximation.
/// see: https://cp-algorithms.com/others/stern_brocot_tree_farey_sequences.html
fn farey(real_number: f64) -> Result<Fraction, DivByZeroError> {
    // Initialize bounds: the target lies between floor(x) and ceil(x)
    let mut left = Fraction::new(real_number.floor() as u64, 1)?;
    let mut right = Fraction::new(real_number.ceil() as u64, 1)?;

    loop {
        // The mediant always lies strictly between left and right (when they differ)
        let mediant = left.mediant(&right)?;
        let mediant_value = mediant.value();

        println!(
            "$ frac({},{}) <- {} -> frac({},{}) $",
            left.numerator, left.denominator,
            mediant_value,
            right.numerator, right.denominator
        );

        // Convergence: mediant is close enough to target
        if (real_number - mediant_value).abs() < f64::EPSILON {
            return Ok(mediant);
        }

        // Binary search: narrow the bounds based on which side the target falls
        if mediant_value > real_number {
            right = mediant;
        } else {
            left = mediant;
        }
    }
}

fn main() {
    let opts = args().run();
    match farey(opts.number) {
        Ok(approx) => println!("{approx}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
