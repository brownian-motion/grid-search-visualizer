use crate::model::*;
use std::collections::{VecDeque};

type GridPos = (usize, usize);

pub struct BreadthFirstSearcher {
    frontier: VecDeque<GridPos>,
}

impl BreadthFirstSearcher {
    pub(crate) fn new(source: GridPos, target: GridPos) -> Self {
        let mut frontier: VecDeque<GridPos> = VecDeque::new();
        frontier.push_front(source);
        BreadthFirstSearcher {
            frontier,
        }
    }
}

impl GridSearchStepper for BreadthFirstSearcher {
    fn step_search(&mut self, grid: &mut Grid) {
        if self.frontier.is_empty() {
            return;
        }

        // TODO: search
    }

    fn reset(&mut self, source: GridPos, target: GridPos) {
        self.frontier.clear();
        self.frontier.push_front(source);
    }
}