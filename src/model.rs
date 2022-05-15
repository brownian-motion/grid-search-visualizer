use std::clone::Clone;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

use druid::{Data, Lens};
use itertools::Itertools;
use rand::{Rng, thread_rng};

use crate::DynamicGridSearcher;

type GridPos = (usize, usize);

#[derive(Clone, Lens, Data)]
pub(crate) struct AppState {
    pub grid_size_slider: f64,
    pub grid: Grid,
    pub paused: bool,
    pub fill_percent: f64,
    pub searcher: Arc<DynamicGridSearcher>,
}

pub trait GridSearchStepper {
    fn step_search(&mut self, grid: &mut Grid) -> bool;
    fn reset(&mut self, source: GridPos, target: GridPos);
}

impl AppState {
    pub fn new(n_rows: usize, fill_percent: f64, searcher: DynamicGridSearcher) -> Self {
        AppState {
            grid_size_slider: n_rows as f64,
            grid: Grid::empty(25, 25),
            paused: true,
            fill_percent,
            searcher: Arc::new(searcher.into()),
        }
    }

    pub fn set_search_endpoints(&mut self, source: GridPos, target: GridPos) {
        self.grid.set_source(source.0, source.1);
        self.grid.set_target(target.0, target.1);
        Arc::make_mut(&mut self.searcher).reset(source, target);
    }

    pub fn fill_randomly(mut self, p: f64) -> Self {
        self.fill_percent = p;
        self.regenerate_grid();
        self
    }

    pub fn regenerate_grid(&mut self) {
        self.paused = true;
        let mut rng = thread_rng();
        self.grid = Grid::empty(self.grid_size_slider as usize, self.grid_size_slider as usize);
        self.grid.regenerate(|_row, _col| rng.gen_bool(self.fill_percent));
        let source_r = (self.grid_size_slider * 0.2) as usize;
        let target_r = (self.grid_size_slider * 0.8) as usize;
        self.set_search_endpoints((source_r, source_r), (target_r, target_r));
    }

    pub fn search_step_delay(&self) -> Duration {
        Duration::from_secs(5).div_f64(self.grid_size_slider.powf(3.0))
    }

    pub fn step_search(&mut self) {
        let done = Arc::make_mut(&mut self.searcher).step_search(&mut self.grid);
        self.paused |= done
    }

    pub fn toggle_paused(&mut self) {
        self.paused = !self.paused;
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
    target_idx: usize,
    source_idx: usize,
}

// TODO: incorporate estimated distance from goal, maybe?
#[derive(Clone, PartialEq)]
pub enum CellState {
    OPEN,
    WALL,
    FRONTIER,
    VISITED,
    SOURCE,
    TARGET,
}

const NEIGHBOR_OFFSETS_8WAY: &'static [(i64, i64); 8] =
    &[(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];

const NEIGHBOR_OFFSETS_4WAY: &'static [(i64, i64); 4] =
    &[(-1, 0), (0, -1), (0, 1), (1, 0)];

impl Grid {
    pub fn empty(n_rows: usize, n_cols: usize) -> Self {
        Grid {
            n_rows,
            n_cols,
            is_wall: vec![false; n_rows * n_cols].into(),
            is_frontier: vec![false; n_rows * n_cols].into(),
            is_visited: vec![false; n_rows * n_cols].into(),
            target_idx: usize::MAX,
            source_idx: usize::MAX,
        }
    }

    pub fn regenerate<T>(&mut self, mut wall_generator: T) -> &mut Self
        where T: FnMut(usize, usize) -> bool
    {
        for row in 0..self.n_rows {
            for col in 0..self.n_cols {
                self.set_wall(row, col, wall_generator(row, col));
            }
        };
        self
    }

    fn rc_to_idx(&self, row: usize, col: usize) -> usize {
        if row >= self.n_rows || col >= self.n_cols {
            return usize::MAX;
        }
        row * self.n_cols + col
    }

    fn idx_to_rc(&self, idx: usize) -> GridPos {
        (idx / self.n_cols, idx % self.n_cols)
    }

    pub fn set_wall(&mut self, row: usize, col: usize, is_wall: bool) {
        self.set_state(row, col, if is_wall { CellState::WALL } else { CellState::OPEN })
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
        let idx = self.rc_to_idx(row, col);
        if idx == self.target_idx {
            CellState::TARGET
        } else if idx == self.source_idx {
            CellState::SOURCE
        } else if self.is_wall[idx] {
            CellState::WALL
        } else if self.is_visited[idx] {
            CellState::VISITED
        } else if self.is_frontier[idx] {
            CellState::FRONTIER
        } else {
            CellState::OPEN
        }
    }

    pub fn set_target(&mut self, row: usize, col: usize) {
        self.target_idx = self.rc_to_idx(row, col)
    }

    pub fn set_source(&mut self, row: usize, col: usize) {
        self.source_idx = self.rc_to_idx(row, col)
    }

    pub fn set_state(&mut self, row: usize, col: usize, state: CellState) {
        use CellState::*;

        let idx = self.rc_to_idx(row, col);

        Arc::make_mut(&mut self.is_wall)[idx] = (state == WALL);
        Arc::make_mut(&mut self.is_visited)[idx] = (state == VISITED);
        Arc::make_mut(&mut self.is_frontier)[idx] = (state == FRONTIER);

        if state == TARGET {
            self.target_idx = idx
        } else if state == SOURCE {
            self.source_idx = idx
        }
    }

    pub fn cell_states<'a>(&'a self) -> impl Iterator<Item=(usize, usize, CellState)> + 'a {
        let coords = (0..(self.n_rows * self.n_cols)).map(|idx| self.idx_to_rc(idx));
        coords.map(|(row, col)| (row, col, self.cell_state(row, col)))
    }

    pub fn is_visited(&self, row: usize, col: usize) -> bool {
        self.is_visited[self.rc_to_idx(row, col)]
    }

    pub fn mark_visited(&mut self, row: usize, col: usize) {
        self.set_state(row, col, CellState::VISITED)
    }

    pub fn is_frontier(&self, row: usize, col: usize) -> bool {
        self.is_frontier[self.rc_to_idx(row, col)]
    }

    pub fn mark_frontier(&mut self, row: usize, col: usize) {
        self.set_state(row, col, CellState::FRONTIER)
    }

    pub fn is_wall(&self, row: usize, col: usize) -> bool {
        self.is_wall[self.rc_to_idx(row, col)]
    }

    pub fn is_source(&self, row: usize, col: usize) -> bool {
        self.source_idx == self.rc_to_idx(row, col)
    }

    pub fn is_target(&self, row: usize, col: usize) -> bool {
        self.target_idx == self.rc_to_idx(row, col)
    }

    pub fn neighbors(&self, row: usize, col: usize) -> Vec<GridPos> {
        NEIGHBOR_OFFSETS_4WAY.into_iter()
            .map(move |(or, oc)| (or + row as i64, oc + col as i64))
            .filter(|&(r, c): &(i64, i64)| r >= 0 && r < self.n_rows as i64 && c >= 0 && c < self.n_cols as i64)
            .map(|(r, c)| (r as usize, c as usize))
            .collect_vec()
    }
}
