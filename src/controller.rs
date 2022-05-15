use std::collections::VecDeque;

use crate::model::*;

type GridPos = (usize, usize);

#[derive(Clone)]
pub enum DynamicGridSearcher {
    BreadthFirst(BreadthFirstSearcher)
    // TODO: add more
}

impl GridSearchStepper for DynamicGridSearcher {
    fn step_search(&mut self, grid: &mut Grid) {
        use DynamicGridSearcher::*;
        match self {
            BreadthFirst(delegate) => delegate.step_search(grid)
        }
    }

    fn reset(&mut self, source: GridPos, target: GridPos) {
        use DynamicGridSearcher::*;
        match self {
            BreadthFirst(delegate) => delegate.reset(source, target)
        }
    }
}

impl From<BreadthFirstSearcher> for DynamicGridSearcher {
    fn from(delegate: BreadthFirstSearcher) -> Self {
        DynamicGridSearcher::BreadthFirst(delegate)
    }
}

#[derive(Clone)]
pub struct BreadthFirstSearcher {
    frontier: VecDeque<GridPos>,
}

impl BreadthFirstSearcher {
    pub(crate) fn new() -> Self {
        BreadthFirstSearcher { frontier: VecDeque::new() }
    }
}

impl GridSearchStepper for BreadthFirstSearcher {
    fn step_search(&mut self, grid: &mut Grid) {
        let (row, col) = match self.frontier.pop_front() {
            Some(coord) => coord,
            None => {
                println!("  we already finished searching");
                return; // we're already done
            }
        };

        if grid.is_target(row, col) {
            println!("  found target");
            self.frontier.clear();
            return;
        }

        if grid.is_visited(row, col) {
            println!("  already visited ({}, {})", row, col);
            return;
        }
        grid.mark_visited(row, col);
        println!("  visiting ({}, {})", row, col);

        for (nr, nc) in grid.neighbors(row, col) {
            if !grid.is_untouched(nr, nc) {
                println!("   skipping neighbor ({}, {})", nr, nc);
                continue;
            }
            println!("   extending frontier to ({}, {})", nr, nc);
            grid.mark_frontier(nr, nc);
            self.frontier.push_back((nr, nc));
        }
    }

    fn reset(&mut self, source: GridPos, target: GridPos) {
        self.frontier.clear();
        self.frontier.push_front(source);
    }
}