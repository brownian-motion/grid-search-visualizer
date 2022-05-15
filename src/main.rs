use std::default::Default;
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};
use druid::widget::{Button, Flex, Label, LabelText};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc, Data, Lens, EventCtx, Event, Env, LifeCycleCtx, LifeCycle, UpdateCtx, LayoutCtx, BoxConstraints, Size, PaintCtx, TimerToken, Color, Point, Rect, RenderContext};
use itertools::Itertools;
use rand::Rng;

#[derive(Clone, Lens, Data)]
struct AppState {
    grid: Grid,
    paused: bool,
    fill_percent: f64,
}

impl AppState {
    pub(crate) fn search_step_delay(&self) -> Duration {
        Duration::from_millis(300)
    }
}

#[allow(clippy::rc_buffer)] // what does this do?
#[derive(Clone, Data, PartialEq)]
struct Grid {
    n_rows: usize,
    n_cols: usize,
    is_wall: Arc<Vec<bool>>,
    is_frontier: Arc<Vec<bool>>,
    is_visited: Arc<Vec<bool>>,
}

// TODO: incorporate estimated distance from goal, maybe?
enum CellState {
    OPEN,
    WALL,
    FRONTIER,
    VISITED,
    GOAL,
    TARGET,
}

impl Grid {
    fn empty(n_rows: usize, n_cols: usize) -> Self {
        Grid {
            n_rows,
            n_cols,
            is_wall: vec![false; n_rows * n_cols].into(),
            is_frontier: vec![false; n_rows * n_cols].into(),
            is_visited: vec![false; n_rows * n_cols].into(),
        }
    }

    fn generate<T>(mut self, mut wall_generator: T) -> Self
        where T: FnMut(usize, usize) -> bool
    {
        for row in 0..self.n_rows {
            for col in 0..self.n_cols {
                self.set_wall(row, col, wall_generator(row, col));
            }
        };
       self
    }

    fn set_wall(&mut self, row: usize, col: usize, is_wall: bool) {
        Arc::make_mut(&mut self.is_wall)[row * self.n_cols + col] = is_wall
    }

    fn clear_visited(&mut self) {
        Arc::make_mut(&mut self.is_frontier).fill(false);
        Arc::make_mut(&mut self.is_visited).fill(false);
    }

    fn clear(&mut self) {
        Arc::make_mut(&mut self.is_wall).fill(false);
        self.clear_visited()
    }

    fn cell_state(&self, row: usize, col: usize) -> CellState {
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

    fn cell_states<'a>(&'a self) -> impl Iterator<Item=(usize, usize, CellState)> +'a{
        let rows = 0..self.n_rows;
        let cols = 0..self.n_cols;
        let coords: itertools::Product<Range<usize>, Range<usize>> = rows.into_iter().cartesian_product(cols);
        coords.map(|(row, col)| (row, col, self.cell_state(row, col)))
    }
}

type ColorScheme = &'static [Color; 6];

const COLORS: ColorScheme = &[
    Color::rgb8(0xFF, 0xFF, 0xFF), // OPEN => white
    Color::rgb8(0x00, 0x00, 0x00), // WALL => black
    Color::rgb8(0x40, 0xD8, 0x10), // VISITED => yellowish green
    Color::rgb8(0xA0, 0xD0, 0x10), // FRONTIER => yellow
    Color::rgb8(0x20, 0xFF, 0x20), // SOURCE => bright green
    Color::rgb8(0xFF, 0x20, 0x20), // TARGET => bright red
];

struct GridWidget {
    timer_id: TimerToken,
    last_update: Instant,
    color_scheme: ColorScheme,
}

impl GridWidget {
    fn new() -> Self {
        GridWidget {
            timer_id: TimerToken::INVALID,
            last_update: Instant::now(),
            color_scheme: COLORS,
        }
    }


    fn schedule_timer(&mut self, ctx: &mut EventCtx, app_state: &AppState) {
        let deadline = app_state.search_step_delay();
        self.last_update = Instant::now();
        self.timer_id = ctx.request_timer(deadline)
    }

    fn cell_color(&self, cell_state: CellState) -> &'static Color {
        use CellState::*;
        match cell_state {
            OPEN => &self.color_scheme[0],
            WALL => &self.color_scheme[1],
            VISITED => &self.color_scheme[2],
            FRONTIER => &self.color_scheme[3],
            SOURCE => &self.color_scheme[4],
            TARGET => &self.color_scheme[5],
        }
    }
}

impl Widget<AppState> for GridWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_paint();
                self.schedule_timer(ctx, data);
            }
            Event::Timer(id) if *id == self.timer_id => {
                if !data.paused {
                    // TODO: step through the grid search
                    ctx.request_paint();
                }
                self.schedule_timer(ctx, data);
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        // nothing to do!
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {
        if data.grid != old_data.grid {
            ctx.request_paint()
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
        let max_side = bc.max();
        let min_side = max_side.height.min(max_side.width);
        // expand to fit parent squarely
        Size {
            width: min_side,
            height: min_side,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        let size: Size = ctx.size();
        let cell_size: Size = Size { width: size.width / (data.grid.n_cols as f64), height: size.height / (data.grid.n_rows as f64) };
        data.grid.cell_states()
            .map(|(row, col, cell_state)| -> (Rect, &'static Color) {
                (
                    Rect::from_origin_size(
                        Point { x: col as f64 * cell_size.width, y: row as f64 * cell_size.height },
                        cell_size,
                    ),
                    self.cell_color(cell_state),
                )
            })
            .for_each(|(rect, color)| ctx.fill(rect, color));
    }
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui)
        .title(LocalizedString::new("app-title").with_placeholder("Grid Search Visualizer"));
    let mut rng = rand::thread_rng();
    let fill_percent = 0.3;
    let app_state: AppState = AppState {
        paused: true,
        grid: Grid::empty(25,25).generate(move |_row, _col| rng.gen_bool(fill_percent)),
        fill_percent,
    }; // todo: expand this
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(app_state)
}

fn build_ui() -> impl Widget<AppState> {
    let grid = GridWidget::new();

    Flex::column().with_flex_child(grid, 1.0)
}
