use druid::{
    text::{TextAlignment, FontFamily}, widget::{Flex, Label, List, Scroll, TextBox, Controller, RawLabel, Container, SizedBox}, commands, AppDelegate, AppLauncher, Command, Data, DelegateCtx, Env, FileDialogOptions, FileInfo, FileSpec, Handled, Lens, LocalizedString, Menu, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc, FontDescriptor, UpdateCtx, theme, Color, EventCtx, Event, LifeCycleCtx, LifeCycle, LayoutCtx, BoxConstraints, Size, PaintCtx, RenderContext, Rect, Point, WidgetPod
};
use std::fs;
use std::sync::Arc;
use ropey::Rope;

#[derive(Clone, Data, Lens)]
struct AppState {
    content: String,
    file_path: Option<String>,
    is_modified: bool,
    cursor_pos: usize,
    #[data(ignore)]
    rope: Rope,
    num_lines: Arc<Vec<usize>>,
}

impl Default for AppState {
    fn default() -> Self {
        let rope = Rope::from_str("");
        Self {
            content: String::new(),
            file_path: None,
            is_modified: false,
            cursor_pos: 0,
            rope,
            num_lines: Arc::new(vec![]),
        }
    }
}

impl AppState {
    fn sync_lines_from_rope(&mut self) {
        self.num_lines = Arc::new((0..self.rope.len_lines()).collect());
    }

    fn sync_from_rope(&mut self) {
        self.content = self.rope.to_string();
    }
    
    fn sync_to_rope(&mut self) {
        let rope_content = self.rope.to_string();
        if rope_content != self.content {
            self.rope = Rope::from_str(&self.content);
            self.is_modified = true;
        }
        self.sync_lines_from_rope();
    }
    
    fn open_file(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.rope = Rope::from_str(&content);
        self.sync_from_rope();
        self.file_path = Some(path.to_string());
        self.is_modified = false;
        self.cursor_pos = 0;
        self.sync_lines_from_rope();
        Ok(())
    }
    
    fn save_file(&mut self, path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let save_path = match path {
            Some(p) => p,
            None => self.file_path.as_ref().ok_or("No file path specified")?,
        };
        
        fs::write(save_path, &self.content)?;
        
        if path.is_some() {
            self.file_path = Some(save_path.to_string());
        }
        
        self.is_modified = false;
        Ok(())
    }
}

const OPEN_FILE_SELECTOR: Selector = Selector::new("file.file-open");
const SAVE_FILE_SELECTOR: Selector = Selector::new("file.file-save");

fn build_ui() -> impl Widget<AppState> {
    let line_numbers = List::new(|| {
        Label::dynamic(|i: &usize, _| format!("{:>4}", i + 1))
            .with_font(FontDescriptor::new(FontFamily::MONOSPACE))
            .with_text_alignment(TextAlignment::End)
            .with_text_color(Color::grey8(100))
    }).lens(AppState::num_lines)
        .fix_width(40.0)
        .padding((5.0, 0.0));
    

    let editor = TextBox::multiline()
        .with_placeholder("")
        .with_font(FontDescriptor::new(FontFamily::MONOSPACE))
        .lens(AppState::content)
        .controller(CodeAreaCallback());

    let editor = editor
        .env_scope(|env, _| {
            env.set(theme::TEXTBOX_BORDER_WIDTH, 0.0);
            env.set(theme::PRIMARY_LIGHT, Color::TRANSPARENT);
            env.set(theme::BACKGROUND_LIGHT, Color::TRANSPARENT);
        });

    let background = Flex::row()
        .with_child(
            SizedBox::empty().fix_width(50.0)
                .background(Color::rgb8(39, 39, 39))
                .expand_height()
        )
        .with_flex_child(
            SizedBox::empty().background(Color::grey8(50))
                .expand(),
            1.0
        )
        .expand();
    Stack::new()
        .with_child(background)
        .with_child(
            Scroll::new(
                Flex::row()
                    .with_child(line_numbers)
                    .with_flex_child(Scroll::new(editor)
                        .horizontal()
                        .content_must_fill(true)
                        .expand_width(),
                    1.0
                )
            ).vertical()
        )
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

pub struct Stack {
    children: Vec<WidgetPod<AppState, Box<dyn Widget<AppState>>>>,
}

impl Stack {
    pub fn new() -> Self {
        Stack { children: vec![] }
    }

    pub fn with_child(mut self, child: impl Widget<AppState> + 'static) -> Self {
        self.children.push(WidgetPod::new(Box::new(child)));
        self
    }
}

impl Widget<AppState> for Stack {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        for child in self.children.iter_mut() {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppState, env: &Env) {
        for child in self.children.iter_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppState, data: &AppState, env: &Env) {
        for child in self.children.iter_mut() {
            child.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &AppState, env: &Env) -> Size {
        let size = bc.max();
        for child in self.children.iter_mut() {
            child.layout(ctx, &BoxConstraints::tight(size), data, env);
            child.set_origin(ctx, Point::ORIGIN);
        }
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppState, env: &Env) {
        for child in self.children.iter_mut() {
            child.paint(ctx, data, env);
        }
    }
}

struct Delegate;
impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: Target,
        cmd: &Command,
        data: &mut AppState,
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

                ctx.submit_command(commands::SHOW_SAVE_PANEL.with(options).to(target));
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get::<FileInfo>(commands::OPEN_FILE) {
            println!("[*] File opened: {:?}", file_info.path());

            match data.open_file(&file_info.path().to_string_lossy()) {
                Ok(()) => {
                    println!("opened");
                },
                Err(_) => {
                    println!("error");
                }
            }
            
            return Handled::Yes;
        }

        if let Some(file_info) = cmd.get::<FileInfo>(commands::SAVE_FILE_AS) {
            println!("[*] File saved to: {:?}", file_info.path());
            return Handled::Yes;
        }

        Handled::No
    }
}

struct CodeAreaCallback();

impl<W: Widget<AppState> + 'static> Controller<AppState, W> for CodeAreaCallback {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        child.event(ctx, event, data, env);
        
        data.sync_to_rope();
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env);
    }
}

fn main() {
    let main_window = WindowDesc::new(build_ui())
        .title("rm-editor")
        .menu(|_window_id, _data, _env| build_menu());

    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .launch(AppState::default())
        .expect("launch failed");
}
