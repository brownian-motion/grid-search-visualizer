use std::sync::Arc;

use druid::{AppLauncher, Env, Event, EventCtx, Lens, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc};
use druid::widget::{Button, Checkbox, Flex, Label, Slider};

use controller::*;
use model::*;
use view::*;

mod model;
mod view;
mod controller;

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui)
        .title(LocalizedString::new("app-title").with_placeholder("Grid Search Visualizer"));
    let mut app_state: AppState = AppState::new(25, 0.3, BreadthFirstSearcher::new().into());
    app_state.regenerate_grid();
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
            data.paused = false;
            ctx.request_paint();
        })
        .padding(5.0);

    let percent_fill_slider = Slider::new()
        .with_range(0.0, 1.0)
        .lens(AppState::fill_percent);
    let percent_fill_label = Label::new(|data: &f64, env: &_| format!("{:02}% fill", (*data * 100.0) as u64).into()).lens(AppState::fill_percent);


    let dimensions_slider = Slider::new()
        .with_range(5.0, 100.0)
        .lens(AppState::grid_size_slider);
    let dimensions_label = Label::new(|data: &AppState, env: &_| format!("{} x {0}", data.grid_size_slider as u64,).into());

    let play_button_text = |data: &AppState, env: &Env| (if data.paused { "Play" } else { "Pause" }).into();
    let play_button = Button::new(play_button_text)
        .on_click(|ctx: &mut EventCtx, data: &mut AppState, _env: &Env| {
            data.toggle_paused();
            ctx.request_update();
        })
        .padding(5.0);

    Flex::row()
        .with_flex_spacer(0.5)
        .with_flex_child(grid.expand_width(), 1.0)
        .with_flex_child(
            Flex::column()
                .with_child(
                    Flex::row()
                        .with_child(dimensions_slider)
                        .with_child(dimensions_label)
                )
                .with_child(
                    Flex::row()
                        .with_child(percent_fill_slider)
                        .with_child(percent_fill_label)
                )
                .with_child(reset_button)
                .with_spacer(50.0)
                .with_child(play_button)
            ,
            0.5,
        )
}
