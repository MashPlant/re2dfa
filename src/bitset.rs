use std::fmt;
use std::iter;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct BitSet {
  inner: Box<[u64]>
}

impl BitSet {
  #[inline]
  pub fn new(n: usize) -> BitSet {
    BitSet { inner: iter::repeat(0).take((n + 63) >> 6).collect() }
  }

  #[inline]
  pub fn from_vec(v: &Vec<bool>) -> BitSet {
    let n = v.len();
    let mut ret = BitSet { inner: iter::repeat(0).take((n + 63) >> 6).collect() };
    unsafe {
      for i in 0..n {
        if *v.get_unchecked(i) {
          ret.set_unchecked(i);
        }
      }
    }
    ret
  }

  #[inline]
  pub fn clear_all(&mut self) {
    for x in self.inner.iter_mut() {
      *x = 0;
    }
  }

  // I just need this bool
  // but it seems that there is not a library that provides it
  #[inline]
  pub fn or(&mut self, other: &BitSet) -> bool {
    let mut changed = false;
    for (x, y) in self.inner.iter_mut().zip(other.inner.iter()) {
      let ox = *x;
      *x |= *y;
      changed |= *x != ox;
    }
    changed
  }

  // it is possible that the n is out of range that `new` specified
  // no check, for my convenience
  #[inline]
  pub fn test(&self, n: usize) -> bool {
    return ((self.inner[n >> 6] >> (n & 63)) & 1) != 0;
  }

  #[inline]
  pub unsafe fn test_unchecked(&self, n: usize) -> bool {
    return ((self.inner.get_unchecked(n >> 6) >> (n & 63)) & 1) != 0;
  }

  #[inline]
  pub fn set(&mut self, n: usize) {
    self.inner[n >> 6] |= (1 as u64) << (n & 63);
  }

  #[inline]
  pub unsafe fn set_unchecked(&mut self, n: usize) {
    *self.inner.get_unchecked_mut(n >> 6) |= (1 as u64) << (n & 63);
  }

  #[inline]
  pub fn clear(&mut self, n: usize) {
    self.inner[n >> 6] &= !((1 as u64) << (n & 63));
  }

  #[inline]
  pub unsafe fn clear_unchecked(&mut self, n: usize) {
    *self.inner.get_unchecked_mut(n >> 6) &= !((1 as u64) << (n & 63));
  }

  #[inline]
  pub fn inner_len(&self) -> usize {
    self.inner.len()
  }

  #[inline]
  pub fn as_ptr(&self) -> *const u64 {
    self.inner.as_ptr()
  }

  #[inline]
  pub fn as_mut_ptr(&mut self) -> *mut u64 {
    self.inner.as_mut_ptr()
  }

  #[inline]
  pub unsafe fn or_raw(mut x: *mut u64, mut y: *const u64, len: usize) -> bool {
    let mut changed = false;
    for _ in 0..len {
      let ox = *x;
      *x |= *y;
      changed |= *x != ox;
      x = x.add(1);
      y = y.add(1);
    }
    changed
  }
}

impl fmt::Debug for BitSet {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let mut l = f.debug_list();
    for i in 0..self.inner.len() * 64 {
      if self.test(i) {
        l.entry(&i);
      }
    }
    l.finish()?;
    Ok(())
  }
}