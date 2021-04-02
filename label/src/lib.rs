use apply::{geom::Point, op::Op};
use geo::{Coordinate, Geometry, Point, Rect};
use neil::Problem;
use std::{cmp::PartialOrd, ops::Range};

mod neil;

pub enum LabelError {
    RulerInsertOutOfRange,
}

#[derive(Debug, Clone)]
pub struct Ruler {
    values: Vec<f64>,
}

impl Ruler {
    fn new(n: usize) -> Ruler {
        Ruler {
            values: vec![1.0; n],
        }
    }

    fn insert(&mut self, range: Range<usize>, score: f64) -> Result<(), LabelError> {
        if range.end > self.values.len() {
            Err(LabelError::RulerInsertOutOfRange)
        } else {
            for index in range {
                self.values[index] *= 1.0 + score;
            }
            Ok(())
        }
    }

    fn score(&self) -> f64 {
        let s: f64 = self.values.iter().sum();
        s - (self.values.len() as f64)
    }
}

#[derive(Clone)]
enum OptimStep {
    Initial,
}

#[derive(Clone)]
pub struct LabelItem {
    // ops: Vec<Op>,
    rect: Rect<f64>,
    geom: Geometry<f64>,
    step: OptimStep,
}

impl LabelItem {
    fn new(rect: Rect<f64>, geom: Geometry<f64>) -> LabelItem {
        LabelItem {
            rect,
            geom,
            step: OptimStep::Initial,
        }
    }

    fn score(&self) -> f64 {
        match self.step {
            OptimStep::Initial => 1.0,
        }
    }
}

pub struct LabelCollection {
    labels: Vec<LabelItem>,
}

const RESOLUTION: usize = 1000;

fn max(a: f64, b: f64) -> f64 {
    match a.partial_cmp(&b) {
        Some(ord) => match ord {
            std::cmp::Ordering::Equal => a,
            std::cmp::Ordering::Greater => a,
            std::cmp::Ordering::Less => b,
        },
        None => a,
    }
}

fn min(a: f64, b: f64) -> f64 {
    match a.partial_cmp(&b) {
        Some(ord) => match ord {
            std::cmp::Ordering::Equal => a,
            std::cmp::Ordering::Greater => b,
            std::cmp::Ordering::Less => a,
        },
        None => a,
    }
}

fn merge_rect(a: &Rect<f64>, b: &Rect<f64>) -> Rect<f64> {
    let a_min = a.min();
    let a_max = a.max();
    let b_min = b.min();
    let b_max = b.max();

    let minx = min(a_min.x, b_min.x);
    let miny = min(a_min.y, b_min.y);
    let maxx = max(a_max.x, b_max.x);
    let maxy = max(a_max.y, b_max.y);

    Rect::new(
        Coordinate { x: minx, y: miny },
        Coordinate { x: maxx, y: maxy },
    )
}

impl LabelCollection {
    fn bbox(&self) -> Rect<f64> {
        if let Some(item) = self.labels.get(0) {
            self.labels
                .iter()
                .skip(1)
                .fold(item.rect.clone(), |acc, item| merge_rect(&acc, &item.rect))
        } else {
            Rect::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 0.0, y: 0.0 })
        }
    }

    fn make_ruler(&self) -> Ruler {
        let bbox = self.bbox();
        let scale = if bbox.height() > bbox.width() {
            bbox.height() / (RESOLUTION as f64)
        } else {
            bbox.width() / (RESOLUTION as f64)
        };

        let south = bbox.min().y;
        let west = bbox.min().x;

        let mut ruler = Ruler::new(2 * RESOLUTION);
        for item in self.labels {
            let minx = ((item.rect.min().x - west) * scale).round() as usize + RESOLUTION;
            let miny = ((item.rect.min().y - south) * scale).round() as usize;
            let maxx = ((item.rect.max().x - west) * scale).round() as usize + RESOLUTION;
            let maxy = ((item.rect.max().y - south) * scale).round() as usize;

            ruler.insert(minx..maxx, item.score());
            ruler.insert(miny..maxy, item.score());
        }

        ruler
    }
}

struct LabelOptim {
    initial: LabelCollection,
}

impl Problem for LabelOptim {
    type State = LabelCollection;

    fn initial_state(&self) -> Self::State {
        self.initial
    }

    fn energy(&self, state: &Self::State) -> f64 {
        state.make_ruler().score() / f64::MAX
    }

    fn new_state(&self, state: &Self::State) -> Self::State {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
