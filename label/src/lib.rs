// use apply::{geom::Point, op::Op};
use geo::{algorithm::translate::Translate, Coordinate, Rect};
use neil::Problem;
use rand::{thread_rng, Rng};
use std::{cmp::PartialOrd, ops::Range};

mod neil;

#[derive(Debug)]
pub enum LabelError {
    RulerInsertOutOfRange,
}

#[derive(Debug, Clone)]
pub struct Ruler {
    values: Vec<f64>,
    max_value: f64,
}

impl Ruler {
    fn new(n: usize) -> Ruler {
        Ruler {
            values: vec![1.0; n],
            max_value: 10_000.0,
        }
    }

    fn insert(&mut self, range: Range<usize>, score: f64) -> Result<(), LabelError> {
        if (range.end) > self.values.len() {
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

    fn to_svg(&self) -> String {
        // let max_value = self.values.iter().fold(0.0, |acc, v| max(acc, v));
        let max_value = self.max_value;
        let scale = self.values.len() as f64 / max_value;
        let rects : String = self.values.iter().enumerate().map(|(i, v)|format!(  r#"<rect x="{}" y="{}" width="{}" height="{}" 
            style="fill:#000000;fill-opacity:1;stroke:#000000;stroke-width:0;stroke-miterlimit:4;stroke-dasharray:none"/>"#,
            i,
            0,
            1,
            v * scale)).collect();
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{}" height="{}"> {} </svg>"#,
            self.values.len(),
            self.values.len(),
            rects
        )
    }
}

#[derive(Clone)]
enum OptimStep {
    TopRight,
    BottomRight,
    TopLeft,
    BottomLeft,
    TopCenter,
    BottomCenter,
}

#[derive(Clone)]
pub struct LabelItem {
    // ops: Vec<Op>,
    rect: Rect<f64>,
    // geom: Geometry<f64>,
    step: OptimStep,
}

impl LabelItem {
    fn new(rect: Rect<f64>, step: OptimStep) -> LabelItem {
        LabelItem {
            rect,
            // geom,
            step,
        }
    }

    fn score(&self) -> f64 {
        match self.step {
            OptimStep::TopRight => 2.0,
            OptimStep::BottomRight => 2.2,
            OptimStep::TopLeft => 2.4,
            OptimStep::BottomLeft => 2.6,
            OptimStep::TopCenter => 2.8,
            OptimStep::BottomCenter => 3.0,
        }
    }

    fn next(&self) -> LabelItem {
        match self.step {
            OptimStep::TopRight => LabelItem::new(
                self.rect.translate(0.0, -self.rect.height()),
                OptimStep::BottomRight,
            ),
            OptimStep::BottomRight => LabelItem::new(
                self.rect.translate(-self.rect.width(), self.rect.height()),
                OptimStep::TopLeft,
            ),
            OptimStep::TopLeft => LabelItem::new(
                self.rect.translate(0.0, -self.rect.height()),
                OptimStep::BottomLeft,
            ),
            OptimStep::BottomLeft => LabelItem::new(
                self.rect
                    .translate(self.rect.width() / 2.0, self.rect.height()),
                OptimStep::TopCenter,
            ),
            OptimStep::TopCenter => LabelItem::new(
                self.rect.translate(0.0, -self.rect.height()),
                OptimStep::BottomCenter,
            ),
            OptimStep::BottomCenter => LabelItem::new(
                self.rect
                    .translate(self.rect.width() / 2.0, self.rect.height()),
                OptimStep::BottomRight,
            ),
        }
    }

    fn to_svg(&self, orig_x: f64, orig_y: f64) -> String {
        format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" 
            style="fill:#000000;fill-opacity:0.1;stroke:#000000;stroke-width:54.04724409;stroke-miterlimit:4;stroke-dasharray:none"/>"#,
            self.rect.min().x - orig_x,
            self.rect.min().y - orig_y,
            self.rect.width(),
            self.rect.height()
        )
    }
}

#[derive(Clone)]
pub struct LabelCollection {
    labels: Vec<LabelItem>,
}

const RESOLUTION: usize = 2000;

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
        let bbox = (self.bbox());
        let scale = if bbox.height() > bbox.width() {
            ((RESOLUTION as f64) / bbox.height())
        } else {
            ((RESOLUTION as f64) / bbox.width())
        };

        let south = bbox.min().y;
        let west = bbox.min().x;

        let mut ruler = Ruler::new(2 * RESOLUTION);
        for item in self.labels.iter() {
            let minx = ((item.rect.min().x - west) * scale).round() as usize + RESOLUTION;
            let miny = ((item.rect.min().y - south) * scale).round() as usize;
            let maxx = ((item.rect.max().x - west) * scale).round() as usize + RESOLUTION;
            let maxy = ((item.rect.max().y - south) * scale).round() as usize;

            ruler.insert(minx..maxx, item.score()).unwrap();
            ruler.insert(miny..maxy, item.score()).unwrap();
        }

        ruler
    }

    fn next(&self) -> LabelCollection {
        let mut rng = thread_rng();
        let mut labels = self.labels.clone();
        let len = self.labels.len();
        let n: usize = rng.gen_range(0..len);
        labels[n] = self.labels[n].next();

        LabelCollection { labels }
    }

    fn to_svg(&self) -> String {
        let bbox = self.bbox();
        let minx = bbox.min().x;
        let miny = bbox.min().y;
        let rects: String = self.labels.iter().map(|i| i.to_svg(minx, miny)).collect();

        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{}" height="{}"> {} </svg>"#,
            bbox.width(),
            bbox.height(),
            rects
        )
    }
}

struct LabelOptim {
    initial: LabelCollection,
}

impl Problem for LabelOptim {
    type State = LabelCollection;

    fn initial_state(&self) -> Self::State {
        self.initial.clone()
    }

    fn energy(&self, state: &Self::State) -> f64 {
        state.make_ruler().score() / f64::MAX
    }

    fn new_state(&self, state: &Self::State) -> Self::State {
        (*state).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neil::Solver;
    use rand::{distributions::Uniform, prelude::Distribution};
    use std::fs::File;
    use std::io::prelude::*;

    fn gen_rect<U>(mut sample: U) -> Rect<f64>
    where
        U: FnMut() -> f64,
    {
        let minx = sample();
        let miny = sample();
        let maxx = min(minx + sample(), minx + 1000.0);
        let maxy = miny + ((maxx - minx) / 2.0);

        dbg!(Rect::new(
            Coordinate { x: minx, y: miny },
            Coordinate { x: maxx, y: maxy },
        ))
    }

    fn gen_col() -> LabelCollection {
        let mut rng = rand::thread_rng();
        let uni = Uniform::from(0.0..10_000.0);
        let uni2 = Uniform::from(0..6);
        let size: usize = 60;
        let mut labels = Vec::with_capacity(size);
        for _ in 0..size {
            labels.push(LabelItem::new(
                gen_rect(|| uni.sample(&mut rng)),
                match uni2.sample(&mut rng) {
                    0 => OptimStep::TopRight,
                    1 => OptimStep::BottomRight,
                    2 => OptimStep::TopLeft,
                    3 => OptimStep::BottomLeft,
                    4 => OptimStep::TopCenter,
                    _ => OptimStep::BottomCenter,
                },
            ));
        }

        LabelCollection { labels }
    }

    fn out(col: &LabelCollection, name: &str) {
        let mut file = File::create(name).unwrap();
        file.write_all(col.to_svg().as_bytes()).unwrap();
        let mut file2 = File::create(format!("ruler-{}", name)).unwrap();
        file2
            .write_all(col.make_ruler().to_svg().as_bytes())
            .unwrap();
    }

    #[test]
    fn it_optimizes() {
        let problem = LabelOptim { initial: gen_col() };
        let solver = Solver::new();
        let solution = solver.solve(&problem);
        let initial_score = problem.initial.make_ruler().score();
        let final_score = solution.make_ruler().score();
        out(&problem.initial, "initial.svg");
        out(&solution, "final.svg");

        assert!(final_score < initial_score);
    }
}
