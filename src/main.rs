use druid::{AppLauncher, Event, EventCtx, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::widget::{Button, Checkbox, Flex, Slider};

use controller::*;
use model::*;
use view::*;

mod model;
mod view;
mod controller;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui)
        .title(LocalizedString::new("app-title").with_placeholder("Grid Search Visualizer"));
    let mut app_state: AppState = AppState::new(25, 25, 0.3, Box::new(BreadthFirstSearcher::new((5, 5), (20, 20))));
    app_state.set_search_endpoints((5, 5), (20, 20));
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(app_state)
}

fn build_ui() -> impl Widget<AppState> {
    let grid = GridWidget::new();

    let reset_button_text = LocalizedString::new("reset-button").with_placeholder("Regenerate");
    let reset_button = Button::new(reset_button_text)
        .on_click(|ctx, data: &mut AppState, _env| {
            data.regenerate_grid();
            ctx.request_paint();
        });

    let percent_fill_slider = Slider::new()
        .with_range(0.0, 1.0)
        .lens(AppState::fill_percent);
    //
    // let play_button = Checkbox::new("Search").on_click(|ctx: &mut EventCtx, data: &mut AppState, _env: &_| {
    //     data.paused = !data.paused;
    // }).lens(AppState::paused);

    Flex::row()
        .with_flex_spacer(0.5)
        .with_flex_child(grid.expand_width(), 1.0)
        .with_flex_child(
            Flex::column()
                .with_child(reset_button)
                .with_child(percent_fill_slider)
                .with_default_spacer()
            // .with_child(play_button)
            ,
            0.5,
        )
}
