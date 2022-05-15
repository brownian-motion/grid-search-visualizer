use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use druid::{Data, Lens};
use itertools::Itertools;

#[derive(Clone, Lens, Data)]
pub(crate) struct AppState {
    pub grid: Grid,
    pub paused: bool,
    pub fill_percent: f64,
}

impl AppState {
    pub fn search_step_delay(&self) -> Duration {
        Duration::from_millis(300)
    }
}

#[allow(clippy::rc_buffer)] // what does this do?
#[derive(Clone, Data, PartialEq)]
pub struct Grid {
    pub n_rows: usize,
    pub n_cols: usize,
    is_wall: Arc<Vec<bool>>,
    is_frontier: Arc<Vec<bool>>,
    is_visited: Arc<Vec<bool>>,
}

// TODO: incorporate estimated distance from goal, maybe?
pub enum CellState {
    OPEN,
    WALL,
    FRONTIER,
    VISITED,
    SOURCE,
    TARGET,
}

impl Grid {
    pub fn empty(n_rows: usize, n_cols: usize) -> Self {
        Grid {
            n_rows,
            n_cols,
            is_wall: vec![false; n_rows * n_cols].into(),
            is_frontier: vec![false; n_rows * n_cols].into(),
            is_visited: vec![false; n_rows * n_cols].into(),
        }
    }

    pub fn generate<T>(mut self, mut wall_generator: T) -> Self
        where T: FnMut(usize, usize) -> bool
    {
        for row in 0..self.n_rows {
            for col in 0..self.n_cols {
                self.set_wall(row, col, wall_generator(row, col));
            }
        };
        self
    }

    pub fn set_wall(&mut self, row: usize, col: usize, is_wall: bool) {
        Arc::make_mut(&mut self.is_wall)[row * self.n_cols + col] = is_wall
    }

    pub fn clear_visited(&mut self) {
        Arc::make_mut(&mut self.is_frontier).fill(false);
        Arc::make_mut(&mut self.is_visited).fill(false);
    }

    pub fn clear(&mut self) {
        Arc::make_mut(&mut self.is_wall).fill(false);
        self.clear_visited()
    }

    pub fn cell_state(&self, row: usize, col: usize) -> CellState {
        let idx = row * self.n_cols + col;
        if self.is_wall[idx] {
            CellState::WALL
        } else if self.is_visited[idx] {
            CellState::VISITED
        } else if self.is_frontier[idx] {
            CellState::FRONTIER
        } else {
            CellState::OPEN
        }
    }

    pub fn cell_states<'a>(&'a self) -> impl Iterator<Item=(usize, usize, CellState)> + 'a {
        let rows = 0..self.n_rows;
        let cols = 0..self.n_cols;
        let coords: itertools::Product<Range<usize>, Range<usize>> = rows.into_iter().cartesian_product(cols);
        coords.map(|(row, col)| (row, col, self.cell_state(row, col)))
    }
}
