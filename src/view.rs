use std::io;
use std::time::Instant;

use druid::{BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, TimerToken, UpdateCtx, Widget};
use rand::Rng;

use crate::model::*;

type ColorScheme = &'static [Color; 6];

const COLORS: ColorScheme = &[
    Color::rgb8(0xFF, 0xFF, 0xFF), // OPEN => white
    Color::rgb8(0x00, 0x00, 0x00), // WALL => black
    Color::rgb8(0x40, 0xD8, 0x10), // VISITED => yellowish green
    Color::rgb8(0xD0, 0xA0, 0x10), // FRONTIER => yellow
    Color::rgb8(0x20, 0xFF, 0x20), // SOURCE => bright green
    Color::rgb8(0xFF, 0x20, 0x20), // TARGET => bright red
];

pub struct GridWidget {
    timer_id: TimerToken,
    last_update: Instant,
    color_scheme: ColorScheme,
}

impl GridWidget {
    pub fn new() -> Self {
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

    pub fn cell_color(&self, cell_state: CellState) -> &'static Color {
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
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, app_state: &mut AppState, _env: &Env) {
        use Event::*;
        match event {
            WindowConnected => {
                ctx.request_paint();
                self.schedule_timer(ctx, app_state);
            }
            Timer(id) if *id == self.timer_id => {
                if !app_state.paused {
                    println!(" stepping search");
                    app_state.step_search();
                    ctx.request_paint();
                }
                self.schedule_timer(ctx, app_state);
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppState, _env: &Env) {
        // nothing to do!
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, _env: &Env) {
        if data.grid != old_data.grid {
            ctx.request_paint()
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &AppState, _env: &Env) -> Size {
        let max_side = bc.max();
        let min_side = max_side.height.min(max_side.width);
        // expand to fit parent squarely
        Size {
            width: min_side,
            height: min_side,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, _env: &Env) {
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
