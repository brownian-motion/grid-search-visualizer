use druid::widget::{Button, Flex, Label, LabelText};
use druid::{AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc, Data, Lens};

#[derive(Clone, Lens, Data)]
struct AppState {
    count: u32,
}

fn main() -> Result<(), PlatformError> {
    let main_window = WindowDesc::new(build_ui)
        .title(LocalizedString::new("app-title").with_placeholder("Grid Search Visualizer"));
    let app_state: AppState = AppState { count: 0_u32 }; // todo: expand this
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(app_state)
}

fn build_ui() -> impl Widget<AppState> {
    let label= Label::new(|data: &u32, _env: &_| format!("Clicks: {}", data)).lens(AppState::count);
    let label = label
        .padding(5.0)
        .center();
    let button = Button::new("increment")
        .on_click(|_ctx, data: &mut AppState, _env| data.count += 1)
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
}
