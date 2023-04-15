use crate::geometry2d::Coordinate;
use crate::r_tree::insert::DimExtremes;

impl<T: Coordinate> DimExtremes<T> {
    pub fn new(low: T, high: T, idx: usize) -> Self {
        Self {
            min_low: low,
            max_low: low,
            min_high: high,
            max_high: high,
            low_idx: idx,
            high_idx: idx,
        }
    }

    pub fn update(&mut self, low: T, high: T, idx: usize) {
        if low < self.min_low {
            self.min_low = low;
        }
        if high > self.max_high {
            self.max_high = high;
        }
        if low > self.max_low {
            self.max_low = low;
            self.low_idx = idx;
        }
        if high > self.min_high {
            self.min_high = high;
            self.high_idx = idx;
        }
    }
}
