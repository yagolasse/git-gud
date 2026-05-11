fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "askpass" {
        let prompt = args.get(2).map(|s| s.as_str()).unwrap_or("Enter passphrase:");
        let port: u16 = std::env::var("GIT_GUD_ASKPASS_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        git_gud::services::askpass::run_client(prompt, port);
        return Ok(());
    }

    let initial_path = if args.len() > 1 && args[1] != "askpass" {
        Some(std::path::PathBuf::from(&args[1]))
    } else {
        None
    };

    run_gui_with_path(initial_path)
}

fn run_gui_with_path(initial_path: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    use eframe::egui;

    git_gud::services::LogService::init()?;
    git_gud::services::git_command::init_config(Default::default());

    let askpass_state = git_gud::services::askpass::AskpassState::new(
        std::sync::Mutex::new(git_gud::services::askpass::AskpassRequests::new()),
    );
    git_gud::services::askpass::set_state(askpass_state.clone());
    let port = git_gud::services::askpass::start_server(askpass_state);

    log::info!("Askpass server listening on port {}", port);

    let mut git_config = git_gud::services::git_command::GitConfig::default();
    git_config.askpass = Some(std::path::PathBuf::from(std::env::current_exe().unwrap_or_default()));
    git_config.askpass_port = Some(port);
    git_gud::services::git_command::init_config(git_config);

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

            Ok(Box::new(GitGudApp::new_with_path(cc, initial_path.clone())))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))
}

struct GitGudApp {
    main_window: git_gud::ui::MainWindow,
}

impl GitGudApp {
    #[allow(dead_code)]
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new_with_path(cc, None)
    }

    fn new_with_path(
        cc: &eframe::CreationContext<'_>,
        initial_path: Option<std::path::PathBuf>,
    ) -> Self {
        log::info!("Creating Git Gud application instance");
        let main_window = if let Some(path) = initial_path {
            git_gud::ui::MainWindow::new_with_path(cc, Some(&path))
        } else {
            git_gud::ui::MainWindow::new(cc)
        };

        Self { main_window }
    }
}

impl eframe::App for GitGudApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.main_window.show(ctx, frame);
    }
}
