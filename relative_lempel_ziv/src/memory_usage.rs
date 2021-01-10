pub struct MemoryUsage {
  reference_size: usize,
  factorizations_size: usize,
  random_access_size: usize,

  raw_size: Option<usize>,
}

impl MemoryUsage {
  pub fn new(
    reference_size: usize,
    factorizations_size: usize,
    random_access_size: usize,
    raw_size: Option<usize>,
  ) -> Self {
    MemoryUsage {
      reference_size,
      factorizations_size,
      random_access_size,
      raw_size,
    }
  }

  pub fn reference_size(&self) -> usize {
    self.reference_size
  }

  pub fn factorizations_size(&self) -> usize {
    self.factorizations_size
  }

  pub fn random_access_size(&self) -> usize {
    self.random_access_size
  }

  pub fn raw_size(&self) -> Option<usize> {
    self.raw_size
  }

  pub fn compressed_size(&self) -> usize {
    self.reference_size + self.factorizations_size
  }

  pub fn total_memory(&self) -> usize {
    self.compressed_size() + self.random_access_size
  }

  pub fn compression_rate(&self) -> Option<f64> {
    match self.raw_size {
      None => None,
      Some(raw_size) => Some(self.total_memory() as f64 / raw_size as f64),
    }
  }

  pub fn compression_rate_without_ra(&self) -> Option<f64> {
    match self.raw_size {
      None => None,
      Some(raw_size) => Some(self.compressed_size() as f64 / raw_size as f64),
    }
  }
}
