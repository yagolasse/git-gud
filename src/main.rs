

mod cli;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 && args[1] != "gui" {
        cli::run()
    } else {
        run_gui()
    }
}

fn run_gui() -> anyhow::Result<()> {
    run_gui_with_path(None)
}

fn run_gui_with_path(initial_path: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    use eframe::egui;
    
    git_gud::services::LogService::init()?;
    
    log::info!("Starting Git Gud GUI application");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Git Gud",
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            Box::new(GitGudApp::new_with_path(cc, initial_path.clone()))
        }),
    ).map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

struct GitGudApp {
    main_window: git_gud::ui::MainWindow,
}

impl GitGudApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_path(cc, None)
    }
    
    fn new_with_path(cc: &eframe::CreationContext<'_>, initial_path: Option<std::path::PathBuf>) -> Self {
        log::info!("Creating Git Gud application instance");
        let main_window = if let Some(path) = initial_path {
            git_gud::ui::MainWindow::new_with_path(cc, Some(&path))
        } else {
            git_gud::ui::MainWindow::new(cc)
        };
        
        Self {
            main_window,
        }
    }
}

impl eframe::App for GitGudApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.main_window.show(ctx, frame);
    }
}
