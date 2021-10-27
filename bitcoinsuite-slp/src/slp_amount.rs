use std::{
    cmp::Ordering,
    iter::Sum,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use bitcoinsuite_core::{BitcoinSuiteError, Result};

use crate::SlpError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlpAmount {
    base_amount: i128,
}

impl SlpAmount {
    pub const ZERO: SlpAmount = SlpAmount::new(0);

    pub fn from_str_decimals(s: &str, decimals: u32) -> Result<Self> {
        let factor = (10i128).pow(decimals);
        let base_amount = match s.find('.') {
            Some(dot_idx) => {
                let integer_part = s[..dot_idx]
                    .parse::<i128>()
                    .map_err(|_| BitcoinSuiteError::NumberParseError)?;
                let integer_part = integer_part * factor;
                let fract_part_str = &s[dot_idx + 1..];
                let preceding_zeros = fract_part_str.chars().take_while(|c| *c == '0').count();
                if fract_part_str.len() > decimals as usize {
                    return Err(BitcoinSuiteError::NumberParseError);
                }
                let num_decimals = fract_part_str.len() as u32;
                let fract_part_str = &fract_part_str[preceding_zeros..];
                if fract_part_str.is_empty() {
                    integer_part
                } else {
                    let fract_part = fract_part_str
                        .parse::<i128>()
                        .map_err(|_| BitcoinSuiteError::NumberParseError)?;
                    let factor = (10i128).pow(decimals - num_decimals);
                    integer_part + fract_part * factor
                }
            }
            None => {
                s.parse::<i128>()
                    .map_err(|_| BitcoinSuiteError::NumberParseError)?
                    * factor
            }
        };
        Ok(SlpAmount { base_amount })
    }

    pub const fn new(base_amount: i128) -> Self {
        SlpAmount { base_amount }
    }

    pub fn from_u64_be(
        slice: &[u8],
        field_name: &'static str,
    ) -> std::result::Result<Self, SlpError> {
        let array: [u8; 8] = slice.try_into().map_err(|_| SlpError::InvalidFieldSize {
            field_name,
            expected: &[8],
            actual: slice.len(),
        })?;
        Ok(SlpAmount {
            base_amount: u64::from_be_bytes(array) as i128,
        })
    }

    pub fn base_amount(&self) -> i128 {
        self.base_amount
    }

    fn _op(&self, other: Self, f: impl Fn(i128, i128) -> i128) -> SlpAmount {
        SlpAmount {
            base_amount: f(self.base_amount, other.base_amount),
        }
    }

    pub fn map(&self, f: impl FnOnce(i128) -> i128) -> SlpAmount {
        SlpAmount {
            base_amount: f(self.base_amount),
        }
    }
}

impl Add for SlpAmount {
    type Output = SlpAmount;

    fn add(self, rhs: SlpAmount) -> Self::Output {
        self._op(rhs, |a, b| a + b)
    }
}

impl AddAssign for SlpAmount {
    fn add_assign(&mut self, rhs: Self) {
        *self = self._op(rhs, |a, b| a + b);
    }
}

impl Sub for SlpAmount {
    type Output = SlpAmount;

    fn sub(self, rhs: SlpAmount) -> Self::Output {
        self._op(rhs, |a, b| a - b)
    }
}

impl SubAssign for SlpAmount {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self._op(rhs, |a, b| a - b);
    }
}

impl Mul<i128> for SlpAmount {
    type Output = SlpAmount;

    fn mul(self, rhs: i128) -> Self::Output {
        self.map(|a| a * rhs)
    }
}

impl MulAssign<i128> for SlpAmount {
    fn mul_assign(&mut self, rhs: i128) {
        *self = self.map(|a| a * rhs);
    }
}

impl Div<i128> for SlpAmount {
    type Output = SlpAmount;

    fn div(self, rhs: i128) -> Self::Output {
        self.map(|a| a / rhs)
    }
}

impl DivAssign<i128> for SlpAmount {
    fn div_assign(&mut self, rhs: i128) {
        *self = self.map(|a| a / rhs);
    }
}

impl Mul<SlpAmount> for i128 {
    type Output = SlpAmount;

    fn mul(self, rhs: SlpAmount) -> Self::Output {
        rhs.map(|a| self * a)
    }
}

impl Div<SlpAmount> for i128 {
    type Output = SlpAmount;

    fn div(self, rhs: SlpAmount) -> Self::Output {
        rhs.map(|a| self / a)
    }
}

impl Neg for SlpAmount {
    type Output = SlpAmount;

    fn neg(self) -> Self::Output {
        SlpAmount {
            base_amount: self.base_amount,
        }
    }
}

impl PartialOrd for SlpAmount {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.base_amount.partial_cmp(&other.base_amount)
    }
}

impl Ord for SlpAmount {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base_amount.cmp(&other.base_amount)
    }
}

impl Sum for SlpAmount {
    fn sum<I: Iterator<Item = SlpAmount>>(mut iter: I) -> Self {
        let mut accumulator = match iter.next() {
            Some(slp_amount) => slp_amount,
            None => {
                return SlpAmount::new(0);
            }
        };
        for val in iter {
            accumulator += val
        }
        accumulator
    }
}

impl std::fmt::Display for SlpAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.base_amount.fmt(f)
    }
}
