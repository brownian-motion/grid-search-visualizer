use druid::{AppLauncher, LocalizedString, PlatformError, Widget, widget::Flex, WindowDesc};
use rand::Rng;

use model::*;
use view::*;

mod model;

mod view;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui)
        .title(LocalizedString::new("app-title").with_placeholder("Grid Search Visualizer"));
    let mut rng = rand::thread_rng();
    let fill_percent = 0.3;
    let app_state: AppState = AppState {
        paused: true,
        grid: Grid::empty(25, 25).generate(move |_row, _col| rng.gen_bool(fill_percent)),
        fill_percent,
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(app_state)
}

fn build_ui() -> impl Widget<AppState> {
    let grid = GridWidget::new();

    Flex::column().with_flex_child(grid, 1.0)
}
