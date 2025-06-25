// main.rs
use druid::{
    AppLauncher, WindowDesc, MenuItem, commands, Command, Data, Env, 
    LocalizedString, Widget, Menu, Selector, AppDelegate, DelegateCtx, Handled, Target, FileDialogOptions, FileSpec, FileInfo
};

#[derive(Clone, Data, Default)]
struct AppState;

const OPEN_FILE_SELECTOR: Selector = Selector::new("file.file-open");
const SAVE_FILE_SELECTOR: Selector = Selector::new("file.file-save");

fn build_ui() -> impl Widget<AppState> {
    druid::widget::Label::new("Hello, Druid!")
}

fn build_menu() -> Menu<AppState> {
    Menu::new("")
        .entry(Menu::empty())
        .entry(file_menu())
}

fn file_menu() -> Menu<AppState> {
    Menu::new(LocalizedString::new("File"))
        .entry(MenuItem::new(LocalizedString::new("Open")).command(OPEN_FILE_SELECTOR))
        .entry(MenuItem::new(LocalizedString::new("Save As")).command(SAVE_FILE_SELECTOR))
        .entry(MenuItem::new(LocalizedString::new("Quit")).command(commands::QUIT_APP))
}

// 커맨드 처리용 AppDelegate
struct Delegate;
impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: Target,
        cmd: &Command,
        _data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        if cmd.is(OPEN_FILE_SELECTOR) {
            let options = FileDialogOptions::new()
                .allowed_types(vec![FileSpec::new("Text files", &["txt", "rs"])])
                .default_type(FileSpec::TEXT)
                .name_label("Select a file")
                .title("Open File");

                ctx.submit_command(commands::SHOW_OPEN_PANEL.with(options).to(target));
            return Handled::Yes;
        }

        if cmd.is(SAVE_FILE_SELECTOR) {
            let options = FileDialogOptions::new()
                .allowed_types(vec![FileSpec::new("Text files", &["txt", "rs"])])
                .default_type(FileSpec::TEXT)
                .name_label("Save file as")
                .title("Save As");

                ctx.submit_command(commands::SHOW_OPEN_PANEL.with(options).to(target));
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get::<FileInfo>(commands::OPEN_FILE) {
            println!("[*] File opened: {:?}", file_info.path());
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get::<FileInfo>(commands::SAVE_FILE_AS) {
            println!("[*] File saved to: {:?}", file_info.path());
            return Handled::Yes;
        }

        Handled::No
    }
}

fn main() {
    let main_window = WindowDesc::new(build_ui())
        .title("Druid File Dialog Example")
        .menu(|_window_id, _data, _env| build_menu());

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .launch(AppState::default())
        .expect("launch failed");
}
