use crate::grid::Coord;
use std::collections::HashSet;
use std::fmt;

#[derive(Debug)]
pub enum BucketQueueError {
    EntropyOutOfBounds { entropy: usize, max: usize },
    ZeroEntropy,
    EntryNotFound { coord: Coord },
}

impl fmt::Display for BucketQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntropyOutOfBounds { entropy, max } => {
                write!(f, "entropy {} exceeds max {}", entropy, max)
            }
            Self::ZeroEntropy => write!(f, "entropy cannot be zero"),
            Self::EntryNotFound { coord } => {
                write!(f, "no entry at {:?}", coord)
            }
        }
    }
}

impl std::error::Error for BucketQueueError {}

pub struct BucketQueue {
    buckets: Vec<HashSet<Coord>>,
}

impl BucketQueue {
    pub fn new(max_entropy: usize) -> Self {
        let buckets = (0..max_entropy).map(|_| HashSet::new()).collect();
        Self { buckets }
    }

    fn get_bucket_index(&self, entropy: usize) -> Result<usize, BucketQueueError> {
        if entropy == 0 {
            return Err(BucketQueueError::ZeroEntropy);
        }
        let index = entropy - 1;
        if index >= self.buckets.len() {
            return Err(BucketQueueError::EntropyOutOfBounds {
                entropy,
                max: self.buckets.len(),
            });
        }
        Ok(index)
    }

    pub fn insert(&mut self, coord: Coord, entropy: usize) -> Result<(), BucketQueueError> {
        let index = self.get_bucket_index(entropy)?;
        self.buckets[index].insert(coord);
        Ok(())
    }

    pub fn update_entropy(
        &mut self,
        coord: Coord,
        new_entropy: usize,
    ) -> Result<(), BucketQueueError> {
        let new_index = self.get_bucket_index(new_entropy)?;

        // Find and remove from whichever bucket it's currently in
        let removed = self.buckets.iter_mut().any(|bucket| bucket.remove(&coord));

        if !removed {
            return Err(BucketQueueError::EntryNotFound { coord });
        }

        self.buckets[new_index].insert(coord);
        Ok(())
    }

    pub fn peek_min(&self) -> Option<Coord> {
        self.buckets
            .iter()
            .find(|bucket| !bucket.is_empty())?
            .iter()
            .next()
            .copied()
    }

    /// Returns (coord, entropy) where entropy is derived from the bucket index
    pub fn extract_min(&mut self) -> Option<(Coord, usize)> {
        let index = self.buckets.iter().position(|bucket| !bucket.is_empty())?;
        let coord = *self.buckets[index].iter().next()?;
        self.buckets[index].remove(&coord);
        Some((coord, index + 1)) // entropy = index + 1
    }

    pub fn remove(&mut self, coord: Coord) -> Result<(), BucketQueueError> {
        let removed = self.buckets.iter_mut().any(|bucket| bucket.remove(&coord));

        if !removed {
            return Err(BucketQueueError::EntryNotFound { coord });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_extract_min() {
        let max_entropy = 10;
        let mut queue = BucketQueue::new(max_entropy);

        queue.insert(Coord::new(0, 0), 5).unwrap();
        queue.insert(Coord::new(1, 1), 3).unwrap();
        queue.insert(Coord::new(2, 2), 7).unwrap();

        let (coord, entropy) = queue.extract_min().expect("Should have an entry");
        assert_eq!(entropy, 3);
        assert_eq!(coord, Coord::new(1, 1));

        let (coord, entropy) = queue.extract_min().expect("Should have another entry");
        assert_eq!(entropy, 5);
        assert_eq!(coord, Coord::new(0, 0));
    }

    #[test]
    fn test_change_entropy() {
        let max_entropy = 15;
        let mut queue = BucketQueue::new(max_entropy);

        queue.insert(Coord::new(0, 0), 5).unwrap();
        queue.insert(Coord::new(1, 1), 8).unwrap();
        queue.insert(Coord::new(2, 2), 14).unwrap();

        queue.update_entropy(Coord::new(1, 1), 2).unwrap();

        let (coord, entropy) = queue.extract_min().expect("Should have an entry");
        assert_eq!(entropy, 2);
        assert_eq!(coord, Coord::new(1, 1));
    }

    #[test]
    fn test_extract_min_empty() {
        let max_entropy = 10;
        let mut queue = BucketQueue::new(max_entropy);

        assert!(queue.extract_min().is_none());

        queue.insert(Coord::new(0, 0), 5).unwrap();
        queue.extract_min();

        assert!(queue.extract_min().is_none());
    }

    #[test]
    fn test_zero_entropy_error() {
        let mut queue = BucketQueue::new(10);
        let result = queue.insert(Coord::new(0, 0), 0);
        assert!(matches!(result, Err(BucketQueueError::ZeroEntropy)));
    }

    #[test]
    fn test_entropy_out_of_bounds_error() {
        let mut queue = BucketQueue::new(5);
        let result = queue.insert(Coord::new(0, 0), 10);
        assert!(matches!(
            result,
            Err(BucketQueueError::EntropyOutOfBounds {
                entropy: 10,
                max: 5
            })
        ));
    }

    #[test]
    fn test_entry_not_found_error() {
        let mut queue = BucketQueue::new(10);
        let result = queue.update_entropy(Coord::new(0, 0), 5);
        assert!(matches!(
            result,
            Err(BucketQueueError::EntryNotFound { .. })
        ));
    }

    #[test]
    fn test_peek_min() {
        let mut queue = BucketQueue::new(10);

        assert!(queue.peek_min().is_none());

        queue.insert(Coord::new(0, 0), 5).unwrap();
        queue.insert(Coord::new(1, 1), 3).unwrap();

        // peek should return the min without removing it
        assert_eq!(queue.peek_min(), Some(Coord::new(1, 1)));
        assert_eq!(queue.peek_min(), Some(Coord::new(1, 1))); // still there
    }
}
