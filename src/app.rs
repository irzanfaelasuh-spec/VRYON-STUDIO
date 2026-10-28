use crate::{editor::{self, EditorTab}, execution::{RunEvent, Runner}, settings::{Settings, ThemeChoice}, workspace};
use eframe::egui::{self, Color32, RichText, Stroke};
use std::{collections::BTreeSet, path::{Path, PathBuf}, time::Duration};

pub struct VyronApp {
    root: Option<PathBuf>,
    files: Vec<workspace::FileEntry>,
    tabs: Vec<EditorTab>,
    active: Option<usize>,
    settings: Settings,
    output: Vec<(bool, String)>,
    runner: Runner,
    status: String,
    search: String,
    workspace_search: String,
    show_search: bool,
    show_outline: bool,
    show_settings: bool,
    show_palette: bool,
    palette_query: String,
    show_output: bool,
    new_name: String,
    expanded: BTreeSet<PathBuf>,
    last_error: Option<String>,
    current_line: usize,
}

impl VyronApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let settings = Settings::load();
        let mut app = Self { root: None, files: vec![], tabs: vec![], active: None, settings, output: vec![],
            runner: Runner::default(), status: "Ready".into(), search: String::new(), workspace_search: String::new(),
            show_search: false, show_outline: true, show_settings: false, show_palette: false, palette_query: String::new(),
            show_output: true, new_name: String::new(), expanded: BTreeSet::new(), last_error: None, current_line: 1 };
        app.apply_theme(&cc.egui_ctx);
        app
    }
    fn apply_theme(&self, ctx: &egui::Context) {
        let mut v = match self.settings.theme { ThemeChoice::Light => egui::Visuals::light(), _ => egui::Visuals::dark() };
        if self.settings.theme != ThemeChoice::Light {
            v.panel_fill = if self.settings.theme == ThemeChoice::Midnight { Color32::from_rgb(15, 18, 27) } else { Color32::from_rgb(27, 29, 34) };
            v.window_fill = Color32::from_rgb(31, 35, 42);
            v.extreme_bg_color = Color32::from_rgb(20, 22, 27);
            v.faint_bg_color = Color32::from_rgb(35, 39, 46);
            v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(48, 54, 64));
            v.selection.bg_fill = Color32::from_rgb(39, 91, 76);
            v.hyperlink_color = Color32::from_rgb(103, 232, 181);
        }
        ctx.set_visuals(v);
    }
    fn refresh(&mut self) { if let Some(r) = &self.root { self.files = workspace::scan(r); } }
    fn open_project(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.root = Some(path.clone()); self.expanded.insert(path.clone()); self.refresh();
            self.status = format!("Workspace: {}", path.display());
        }
    }
    fn open_file(&mut self, path: PathBuf) {
        if let Some(i) = self.tabs.iter().position(|t| t.path == path) { self.active = Some(i); return; }
        match workspace::read(&path) {
            Ok(text) => { self.tabs.push(EditorTab::new(path, text)); self.active = Some(self.tabs.len()-1); self.status = "File opened".into(); }
            Err(e) => self.notify(e.to_string()),
        }
    }
    fn save_active(&mut self) {
        if let Some(i) = self.active {
            let path = self.tabs[i].path.clone(); let text = self.tabs[i].text.clone();
            match workspace::write(&path, &text) {
                Ok(()) => { self.tabs[i].saved_text = text; self.status = format!("Saved {}", path.display()); }
                Err(e) => self.notify(e.to_string()),
            }
        }
    }
    fn notify(&mut self, s: String) { self.last_error = Some(s.clone()); self.output.push((true, s.clone())); self.status = s; }
    fn import_file(&mut self) {
        let Some(root) = self.root.clone() else { self.notify("Open a workspace before importing files.".into()); return; };
        if let Some(source) = rfd::FileDialog::new().pick_file() {
            match workspace::import_file(&source, &root) { Ok(dest) => { self.refresh(); self.open_file(dest); }, Err(e) => self.notify(e.to_string()) }
        }
    }
    fn run_active(&mut self) {
        let Some(i) = self.active else { self.notify("Open a source file before running.".into()); return; };
        if self.tabs[i].dirty() { self.save_active(); }
        let path = self.tabs[i].path.clone();
        let Some(root) = self.root.clone() else { self.notify("Open a workspace folder first.".into()); return; };
        self.output.push((false, format!("$ Run {}", path.display())));
        match self.runner.start(&path, &root) { Ok(()) => { self.status = "Running".into(); self.show_output = true; }, Err(e) => self.notify(e) }
    }
    fn command(&mut self, cmd: &str) {
        match cmd {
            "Open Folder" => self.open_project(), "Import File" => self.import_file(),
            "Save" => self.save_active(), "Run" => self.run_active(),
            "Stop" => { if self.runner.stop() { self.output.push((true, "Process stopped.".into())); self.status = "Stopped".into(); } },
            "Search Workspace" => self.show_search = true,
            "Toggle Outline" => self.show_outline = !self.show_outline,
            "Toggle Output" => self.show_output = !self.show_output,
            "Settings" => self.show_settings = !self.show_settings,
            "New File" => self.create_new(false), "New Folder" => self.create_new(true),
            _ => {}
        }
    }
    fn create_new(&mut self, folder: bool) {
        let Some(root) = self.root.clone() else { self.notify("Open a workspace folder first.".into()); return; };
        let path = if let Some(p) = rfd::FileDialog::new().set_directory(&root).set_file_name(if folder {"new-folder"} else {"untitled.txt"}).save_file() { p } else { return; };
        let result = if folder { workspace::create_dir(&path) } else { workspace::create_file(&path) };
        match result { Ok(()) => { self.refresh(); if !folder { self.open_file(path); } }, Err(e) => self.notify(e.to_string()) }
    }
    fn draw_explorer(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("EXPLORER").strong().size(11.0).color(Color32::from_rgb(147,158,173)));
            if ui.small_button("+").on_hover_text("New file").clicked() { self.create_new(false); }
            if ui.small_button("▣").on_hover_text("New folder").clicked() { self.create_new(true); }
        });
        ui.separator();
        if self.root.is_none() {
            ui.add_space(8.0); ui.label("No folder open");
            if ui.button("Open Folder").clicked() { self.open_project(); }
            return;
        }
        if let Some(root) = self.root.clone() {
            ui.label(RichText::new(root.file_name().unwrap_or_default().to_string_lossy()).strong());
        }
        let entries = self.files.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in entries {
                let name = entry.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let indent = entry.depth as f32 * 10.0;
                ui.horizontal(|ui| {
                    ui.add_space(indent);
                    if entry.is_dir {
                        let open = self.expanded.contains(&entry.path);
                        if ui.small_button(if open {"▾"} else {"▸"}).clicked() {
                            if open { self.expanded.remove(&entry.path); } else { self.expanded.insert(entry.path.clone()); }
                        }
                        if ui.selectable_label(false, format!("▰ {name}")).clicked() {
                            if open { self.expanded.remove(&entry.path); } else { self.expanded.insert(entry.path.clone()); }
                        }
                    } else {
                        let active = self.active.and_then(|i| self.tabs.get(i)).map(|t| t.path == entry.path).unwrap_or(false);
                        let icon = match entry.path.extension().and_then(|e| e.to_str()).unwrap_or("") { "rs"=>"R", "py"=>"Py", "js"|"ts"=>"JS", "json"=>"{}", "md"=>"M", _=>"·" };
                        if ui.selectable_label(active, format!("{icon}  {name}")).clicked() { self.open_file(entry.path.clone()); }
                    }
                });
            }
        });
    }
    fn draw_editor(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal_wrapped(|ui| {
            for i in 0..self.tabs.len() {
                let title = self.tabs[i].title();
                if ui.selectable_label(self.active == Some(i), title).clicked() { self.active = Some(i); }
                if ui.small_button("×").on_hover_text("Close tab").clicked() {
                    if self.tabs[i].dirty() { self.notify(format!("{} has unsaved changes; save before closing.", self.tabs[i].path.display())); }
                    else { self.tabs.remove(i); self.active = if self.tabs.is_empty() { None } else { Some(i.min(self.tabs.len()-1)) }; break; }
                }
            }
        });
        ui.separator();
        let Some(i) = self.active else {
            ui.vertical_centered(|ui| { ui.add_space(90.0); ui.heading(RichText::new("VYRON").size(34.0).color(Color32::from_rgb(103,232,181))); ui.label("Open a folder or create a file to begin."); ui.add_space(12.0); if ui.button("Open Folder").clicked() { self.open_project(); } });
            return;
        };
        let mut layouter = |ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut layout = egui_extras::syntax_highlighting::highlight(
                &egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx()),
                text.as_str(), editor::language_for(&self.tabs[i].path)
            );
            layout.wrap.max_width = if self.settings.word_wrap { wrap_width } else { f32::INFINITY };
            ui.fonts_mut(|f| f.layout_job(layout))
        };
        let line_count = editor::line_count(&self.tabs[i].text);
        ui.horizontal(|ui| {
            egui::ScrollArea::vertical().id_salt("line_numbers").max_width(45.0).show(ui, |ui| {
                for n in 1..=line_count { ui.label(RichText::new(format!("{n:>4}")).monospace().size(self.settings.font_size-1.0).color(editor::line_number_color())); }
            });
            egui::ScrollArea::both().id_salt("code_editor_scroll").show(ui, |ui| {
                let edit = egui::TextEdit::multiline(&mut self.tabs[i].text).font(egui::FontId::monospace(self.settings.font_size))
                    .code_editor().desired_width(f32::INFINITY).desired_rows(30).lock_focus(true).layouter(&mut layouter);
                ui.add_sized(ui.available_size(), edit);
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.command) { self.save_active(); }
    }
    fn draw_output(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("OUTPUT").strong().size(11.0));
            if ui.small_button("Clear").clicked() { self.output.clear(); }
            if ui.small_button("Copy").clicked() {
                let text = self.output.iter().map(|(_,s)| s.as_str()).collect::<Vec<_>>().join("\n");
                ui.ctx().copy_text(text);
            }
            if self.runner.running { ui.label(RichText::new("● RUNNING").color(Color32::from_rgb(103,232,181))); }
        });
        ui.separator();
        egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
            for (is_err, line) in &self.output {
                ui.label(RichText::new(line).monospace().size(12.0).color(if *is_err { Color32::from_rgb(255,126,126) } else { Color32::from_rgb(204,213,224) }));
            }
        });
    }
    fn draw_outline(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("OUTLINE").strong().size(11.0).color(Color32::from_rgb(147,158,173)));
        ui.separator();
        if let Some(tab) = self.active.and_then(|i| self.tabs.get(i)) {
            let ext = tab.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let symbols = editor::outline(&tab.text, ext);
            if symbols.is_empty() { ui.label("No symbols detected."); }
            for (line, name) in symbols {
                if ui.small_button(format!("{}  {}", line, name)).clicked() { self.current_line = line; self.status = format!("Symbol at line {line}"); }
            }
        } else { ui.label("Open a source file to view symbols."); }
    }
    fn draw_settings(&mut self, ctx: &egui::Context) {
        let mut open = self.show_settings;
        egui::Window::new("VYRON Settings").open(&mut open).resizable(true).show(ctx, |ui| {
            ui.heading("Appearance");
            egui::ComboBox::from_label("Theme").selected_text(match self.settings.theme { ThemeChoice::Graphite=>"Graphite Dark", ThemeChoice::Midnight=>"Midnight", ThemeChoice::Light=>"Light" })
                .show_ui(ui, |ui| { ui.selectable_value(&mut self.settings.theme, ThemeChoice::Graphite, "Graphite Dark"); ui.selectable_value(&mut self.settings.theme, ThemeChoice::Midnight, "Midnight"); ui.selectable_value(&mut self.settings.theme, ThemeChoice::Light, "Light"); });
            ui.add(egui::Slider::new(&mut self.settings.font_size, 11.0..=24.0).text("Editor font size"));
            ui.checkbox(&mut self.settings.word_wrap, "Word wrap");
            ui.checkbox(&mut self.settings.autosave, "Auto-save (reserved; manual save remains default)");
            ui.checkbox(&mut self.settings.animations, "Animations");
            if ui.button("Save settings").clicked() { self.settings.save(); self.apply_theme(ctx); self.status = "Settings saved".into(); }
        });
        self.show_settings = open;
    }
    fn draw_palette(&mut self, ctx: &egui::Context) {
        let mut open = self.show_palette;
        egui::Window::new("Command Palette").open(&mut open).collapsible(false).resizable(false).default_width(440.0).show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.palette_query);
            let commands = ["Open Folder","Import File","New File","New Folder","Save","Run","Stop","Search Workspace","Toggle Outline","Toggle Output","Settings"];
            for c in commands.iter().filter(|c| c.to_lowercase().contains(&self.palette_query.to_lowercase())) {
                if ui.selectable_label(false, *c).clicked() { self.command(c); self.palette_query.clear(); open = false; break; }
            }
        });
        self.show_palette = open;
    }
}

impl eframe::App for VyronApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_theme(ctx);
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) { self.save_active(); }
        if ctx.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::P)) { self.show_palette = true; }
        if ctx.input(|i| i.key_pressed(egui::Key::F) && i.modifiers.command) { self.show_search = true; }
        for event in self.runner.poll() {
            match event {
                RunEvent::Output(s) => self.output.push((false,s)),
                RunEvent::Error(s) => self.output.push((true,s)),
                RunEvent::Finished(code, secs) => { self.output.push((code != 0, format!("Process exited with code {code} ({secs:.2}s)"))); self.status = if code == 0 {"Completed".into()} else {"Failed".into()}; }
            }
        }
        egui::TopBottomPanel::top("title_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("V").strong().size(22.0).color(Color32::from_rgb(103,232,181)));
                ui.label(RichText::new("VYRON").strong().size(15.0));
                ui.separator();
                ui.label(self.root.as_ref().and_then(|p| p.file_name()).unwrap_or_default().to_string_lossy());
                ui.add_space(12.0);
                if ui.button("Open Folder").clicked() { self.open_project(); }
                if ui.button("Import File").clicked() { self.import_file(); }
                if ui.button("Save").clicked() { self.save_active(); }
                if ui.button("Run").clicked() { self.run_active(); }
                if ui.add_enabled(self.runner.running, egui::Button::new("Stop")).clicked() { self.runner.stop(); self.output.push((true,"Process stopped.".into())); self.status = "Stopped".into(); }
                if ui.button("Settings").clicked() { self.show_settings = true; }
            });
        });
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").color(Color32::from_rgb(103,232,181)));
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Ln {}", self.current_line));
                    if let Some(i) = self.active { ui.label(editor::language_for(&self.tabs[i].path)); }
                    ui.label("UTF-8");
                });
            });
        });
        egui::SidePanel::left("activity").resizable(false).exact_width(46.0).show(ctx, |ui| {
            for (label, tip) in [("▤","Explorer"),("⌕","Search workspace"),("◇","Outline"),("⚙","Settings")] {
                if ui.add_sized([36.0,40.0], egui::Button::new(RichText::new(label).size(22.0))).on_hover_text(tip).clicked() {
                    match tip { "Explorer" => {}, "Search workspace" => self.show_search = !self.show_search, "Outline" => self.show_outline = !self.show_outline, _ => self.show_settings = true }
                }
            }
        });
        egui::SidePanel::left("explorer").resizable(true).default_width(240.0).width_range(175.0..=360.0).show(ctx, |ui| self.draw_explorer(ui));
        if self.show_outline { egui::SidePanel::right("outline").resizable(true).default_width(250.0).width_range(180.0..=400.0).show(ctx, |ui| self.draw_outline(ui)); }
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_search {
                ui.horizontal(|ui| { ui.label("Workspace search"); ui.text_edit_singleline(&mut self.workspace_search); if ui.button("Search").clicked() { self.output.clear(); if let Some(root)=&self.root { for e in workspace::scan(root).into_iter().filter(|e| !e.is_dir) { if let Ok(t)=workspace::read(&e.path) { for (n,line) in t.lines().enumerate() { if line.to_lowercase().contains(&self.workspace_search.to_lowercase()) && !self.workspace_search.is_empty() { self.output.push((false,format!("{}:{}: {}",e.path.display(),n+1,line.trim()))); } } } } self.show_output=true; } } if ui.button("×").clicked(){self.show_search=false;} });
                ui.separator();
            }
            self.draw_editor(ui, ctx);
        });
        if self.show_output {
            egui::TopBottomPanel::bottom("output_panel").resizable(true).default_height(180.0).height_range(80.0..=380.0).show(ctx, |ui| self.draw_output(ui));
        }
        self.draw_settings(ctx);
        self.draw_palette(ctx);
        if let Some(err) = self.last_error.clone() {
            egui::Window::new("VYRON message").collapsible(false).resizable(false).show(ctx, |ui| { ui.label(err); if ui.button("Dismiss").clicked() { self.last_error = None; } });
        }
        if self.settings.animations { ctx.request_repaint_after(Duration::from_millis(200)); }
    }
}
