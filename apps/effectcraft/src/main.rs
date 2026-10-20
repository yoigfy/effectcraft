//! Tokeffects desktop app.
//!
//! Usage: `effectcraft [--control <port>] [--demo|--empty|--home] [--] [project.ecproj | media files…]`
//!
//! An unknown option or a bad port prints an error and exits with status 1 before anything starts.
//!
//! Without a project the app starts with an empty Untitled Project, as After Effects does;
//! `--demo` opens the demo project instead (also Help ▸ Open Demo Project).
//!
//! `--control <port>` (or `TOKCRAFT_CONTROL_PORT`) starts a localhost JSON-lines control server;
//! see `effectcraft_ui_egui::control` for the methods.
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

// Built everywhere so its tests run on every platform; only Linux AppImages use it.
mod appearance;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
mod appimage;
#[cfg(target_os = "macos")]
mod apple_events;
mod audio_out;
mod control_server;
mod launch_guard;
#[cfg(target_os = "macos")]
mod native_menu;
// Built everywhere so it stays checked; the macOS and Linux apps use it.
#[cfg_attr(not(any(target_os = "macos", target_os = "linux")), allow(dead_code))]
mod sheet_dialogs;
mod window;

use effectcraft_ui_egui::EffectcraftApp;
use serde_json::json;

/// Tokeffects's app icon for the window, taskbar and Dock.
const ICON_PNG: &[u8] = include_bytes!("../../../assets/app-icon/tokeffects.png");

fn app_icon() -> egui::IconData {
    let png: &[u8] = ICON_PNG;
    eframe::icon_data::from_png_bytes(png).unwrap_or_default()
}

/// Both native JSON projects and Save a Copy As XML projects are openable on launch.
/// Keep this classification shared by project opening and the subsequent media-import pass.
fn is_project_file(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ecproj") || ext.eq_ignore_ascii_case("ecprojx"))
}

/// winit 0.30 cannot deliver native Wayland file drops. XWayland can.
#[cfg(target_os = "linux")]
fn wayland_drop_notice(wayland_display: Option<&std::ffi::OsStr>, backend: Option<&std::ffi::OsStr>) -> Option<&'static str> {
    (wayland_display.is_some_and(|v| !v.is_empty()) && backend != Some(std::ffi::OsStr::new("x11")))
        .then_some("Native Wayland file drops are unavailable. Use File ▸ Import ▸ File… or run via XWayland (see README, Linux).")
}

/// The command line.
#[derive(Debug, Default, PartialEq)]
struct Args {
    control_port: Option<u16>,
    files: Vec<String>,
    /// Open the demo project when no project is given (`--demo`).
    demo: bool,
    /// Show or skip the Home screen (`--home`), else the Startup preference decides.
    home: Option<bool>,
}

/// Parse the arguments after the program name; `Ok(None)` for `--version`. `env_port` is
/// `TOKCRAFT_CONTROL_PORT`, used when `--control` isn't given. An unknown option or a bad port
/// is an error, so it is neither imported as a file nor dropped; after `--` every argument is a file.
fn parse_args(args: impl IntoIterator<Item = String>, env_port: Result<String, std::env::VarError>) -> Result<Option<Args>, String> {
    let mut out = Args::default();
    let mut args = args.into_iter();
    let mut options = true;
    while let Some(a) = args.next() {
        if !options {
            out.files.push(a);
            continue;
        }
        match a.as_str() {
            "--" => options = false,
            "--control" => out.control_port = Some(control_port("--control", args.next().as_deref())?),
            "--demo" => out.demo = true,
            "--empty" => out.demo = false,
            "--home" => out.home = Some(true),
            "--version" => return Ok(None),
            _ => match a.strip_prefix("--control=") {
                Some(v) => out.control_port = Some(control_port("--control", Some(v))?),
                // macOS LaunchServices may pass a process serial number (`-psn_0_1234`), notably on
                // the first launch of a downloaded app; it is not an option of ours.
                None if a.starts_with("-psn_") => {}
                None if a.starts_with('-') && a != "-" => return Err(format!("unknown option `{a}`")),
                None => out.files.push(a),
            },
        }
    }
    if out.control_port.is_none() {
        out.control_port = match env_port {
            Ok(v) if v.is_empty() => None,
            Ok(v) => Some(control_port("TOKCRAFT_CONTROL_PORT", Some(&v))?),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(v)) => Some(control_port("TOKCRAFT_CONTROL_PORT", Some(&v.to_string_lossy()))?),
        };
    }
    Ok(Some(out))
}

/// A control-channel port from 1 to 65535 (0 would listen on a port nobody is told); `name` is
/// the flag or variable it came from.
fn control_port(name: &str, value: Option<&str>) -> Result<u16, String> {
    let need = format!("{name} needs a port number from 1 to 65535");
    match value {
        None => Err(need),
        Some(v) => v.parse().ok().filter(|&p| p != 0).ok_or(format!("{need}, not `{v}`")),
    }
}

/// The commands' file dialogs open without blocking the frame where the host's backend would:
/// on macOS as sheets instead of nested modal loops (#511), on Linux through the desktop
/// portal from a worker thread — a blocking portal dialog leaves the main thread waiting on its
/// answer without pumping events, so the window stops answering the compositor and the desktop
/// offers to force-quit it while the Open dialog is up (#1275). Windows' blocking dialog pumps
/// its own messages, so it keeps [`effectcraft_ui_egui::Hooks::pick_files`] and friends.
fn install_file_dialogs(app: &mut EffectcraftApp, ctx: &egui::Context) {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let ctx = ctx.clone();
        app.hooks.file_dialog_async = Some(Box::new(move |dialog, done| sheet_dialogs::show(dialog, done, ctx.clone())));
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let _ = (app, ctx);
}

fn main() -> eframe::Result {
    // First, so the panic hook and every start-up warning are recorded. Records go to stderr
    // and the session log; Help ▸ Enable Logging writes through this logger too, and warnings
    // feed the compatibility report.
    effectcraft_engine::logging::install();
    effectcraft_engine::logging::install_panic_hook();
    let Args { control_port, files, demo, home } = match parse_args(std::env::args().skip(1), std::env::var("TOKCRAFT_CONTROL_PORT")) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("Tokeffects {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        // Before the session log, preferences and the window: nothing has started yet.
        Err(e) => {
            eprintln!("Tokeffects: {e}");
            std::process::exit(1);
        }
    };
    // The session log lives in the settings folder (`Logs/tokeffects.log`, the previous two runs
    // kept): a launch from the Dock or a menu has no terminal. Opened after the arguments, so
    // `--version` leaves no file behind; records logged until now are written to it first.
    if let Some(dir) = effectcraft_host::config_dir() {
        match effectcraft_engine::logging::attach_session_log(&effectcraft_engine::logging::log_dir(Some(&dir))) {
            Ok(path) => log::info!("Tokeffects {}, log file {}", env!("CARGO_PKG_VERSION"), path.display()),
            Err(e) => log::warn!("no log file: {e}"),
        }
    }
    // Wayland shows the window's icon from its desktop entry: an AppImage brings its own.
    #[cfg(target_os = "linux")]
    appimage::integrate_from_env(ICON_PNG);
    // Settings ▸ Startup & Repair ▸ Window Graphics, switched to OpenGL when the last launch
    // stopped as its window opened (a crashing graphics driver, #243).
    let mut launch = launch_guard::Launch::begin(effectcraft_host::config_dir().as_deref(), std::env::var_os("WGPU_BACKEND").is_some());
    let on_gl = launch.backends == Some(eframe::wgpu::Backends::GL);
    let notice = launch.notice.take();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tokeffects")
            .with_inner_size(window::DEFAULT_SIZE)
            .with_min_inner_size(window::MIN_SIZE)
            .with_drag_and_drop(true)
            // Started for an agent (`--control`): open without taking the user's keyboard focus.
            .with_active(control_port.is_none())
            .with_icon(app_icon())
            // Wayland matches the window to `ai.storyteller.effectcraft.desktop` by this ID.
            .with_app_id(appimage::APP_ID),
        event_loop_builder: agent_event_loop(control_port.is_some()),
        wgpu_options: wgpu_options(launch.backends),
        ..Default::default()
    };
    // Start-up milestones go to stderr and the session log, so a window that never appears
    // shows how far start-up got (#234).
    log::info!("Tokeffects {}: opening the window", env!("CARGO_PKG_VERSION"));
    // Registered before the event loop starts, so it catches the Finder event that launched us as
    // well as later ones. Lives until the event loop returns; the app creator only borrows it.
    #[cfg(target_os = "macos")]
    let apple_events = apple_events::AppleEvents::install();
    #[cfg(target_os = "macos")]
    let apple_events = &apple_events;
    let result = eframe::run_native(
        "Tokeffects",
        options,
        Box::new(move |cc| {
            match &cc.wgpu_render_state {
                Some(rs) => log::info!("graphics: {:?}", rs.adapter.get_info()),
                None => log::info!("graphics: no wgpu device"),
            }
            // This executable owns the shared device's handlers. Install them before
            // Tokeffects's compositor pipelines are built, without changing handlers from
            // inside Gpu::new. eframe has already constructed its presentation renderer here.
            let gpu_failures = effectcraft_ui_egui::gpu_failure::GpuFailureBridge::new(&cc.egui_ctx);
            if let Some(rs) = &cc.wgpu_render_state {
                let errors = gpu_failures.clone();
                rs.device.on_uncaptured_error(std::sync::Arc::new(move |error| {
                    let message = format!("uncaptured GPU error: {error}");
                    if errors.report(&message, false) {
                        log::error!("{message}");
                    }
                }));
                let lost = gpu_failures.clone();
                rs.device.set_device_lost_callback(move |reason, message| {
                    lost.report(&format!("GPU device lost ({reason:?}): {message}"), true);
                });
            }
            // Restore saved geometry before loading the project so the window can appear
            // right away. The heavier session, plugins and media setup happens off the UI thread.
            let saved_window = effectcraft_host::config_dir().and_then(|dir| {
                let config = effectcraft_engine::config::DirConfig::new(dir);
                window::load(&config)
            });
            let opened = cc.winit_window().map(|w| window::restore(w, saved_window));
            let (startup_tx, startup_rx) = std::sync::mpsc::sync_channel(1);
            std::thread::Builder::new()
                .name("tokeffects-startup".into())
                .spawn(move || {
                    let mut session = effectcraft_host::session();
                    session.check_footage_on_open = true;
                    session.autosave.background = true;

                    if let Some(dir) = effectcraft_host::config_dir() {
                        let plugins = dir.join("Plug-ins");
                        if plugins.is_dir() {
                            let _ = session.execute("effect.plugins.load", json!({"folder": plugins.to_string_lossy()}));
                        }
                        session.config = Some(std::sync::Arc::new(effectcraft_engine::config::DirConfig::new(dir)));
                    }
                    session.load_settings();

                    let show_home = home.unwrap_or(
                        session.prefs.startup.show_home_on_launch && files.is_empty() && control_port.is_none(),
                    );
                    let project = files.iter().find(|f| is_project_file(f)).cloned();
                    if let Some(p) = project {
                        if let Err(e) = session.execute("file.open", json!({"path": p})) {
                            log::warn!("{p}: {e}");
                        }
                    } else if demo {
                        let _ = session.execute("file.openDemoProject", json!({}));
                    }
                    let media: Vec<String> = files.iter().filter(|f| !is_project_file(f)).cloned().collect();
                    if !media.is_empty()
                        && let Err(e) = session.execute("file.import", json!({"paths": media}))
                    {
                        log::warn!("import: {e}");
                    }
                    let _ = session.execute("window.restoreScriptPanels", json!({}));
                    let _ = startup_tx.send(PreparedStartup { session, show_home });
                })?;

            log::info!("Tokeffects: showing splash while the workspace initializes");
            Ok(Box::new(Desktop {
                #[cfg(target_os = "macos")]
                menu: native_menu::NativeBar::new(&cc.egui_ctx),
                #[cfg(target_os = "macos")]
                apple_events: apple_events.connect(&cc.egui_ctx),
                app: None,
                startup_rx: Some(startup_rx),
                startup_error: None,
                gpu_failures,
                notice,
                control_port,
                logo: None,
                window: window::Tracker::new(opened, saved_window, control_port.is_none()),
                launch,
                drawn: false,
            }))
        }),
    );
    match &result {
        Ok(()) => log::info!("the window closed"),
        Err(e) => {
            if on_gl {
                launch_guard::opengl_failed(effectcraft_host::config_dir().as_deref());
            }
            startup_failed(e, !on_gl && cfg!(not(target_os = "macos")));
        }
    }
    result
}

/// The window's wgpu device asks for the adapter's own limits (textures up to 16384 px)
/// instead of eframe's fixed set: an older GPU that can't meet those (Intel HD Graphics 5500,
/// #198) still opens the app, and the compositor then checks what it needs and falls back to
/// the CPU when the device can't run it. `backends`: Window Graphics' choice, when it makes one.
fn wgpu_options(backends: Option<eframe::wgpu::Backends>) -> eframe::WgpuConfiguration {
    use eframe::egui_wgpu::WgpuSetup;
    let mut config = eframe::WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        if let Some(b) = backends {
            setup.instance_descriptor.backends = b;
        }
        // Nothing draws or dispatches indirectly, so wgpu's check of indirect arguments only costs
        // a compute shader when the device is created, one that Metal can't compile on an Intel
        // Iris Pro: the device is lost and the window never opens.
        // `WGPU_VALIDATION_INDIRECT_CALL=1` turns the check back on.
        setup.instance_descriptor.flags = setup.instance_descriptor.flags.difference(eframe::wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL).with_env();
        setup.device_descriptor = std::sync::Arc::new(|adapter| eframe::wgpu::DeviceDescriptor {
            label: Some("Tokeffects device"),
            required_limits: device_limits(adapter.limits()),
            ..Default::default()
        });
    }
    config
}

/// The limits to request from an adapter that offers `adapter`: all of them, with textures
/// capped at 16384 px like the headless renderer's device.
fn device_limits(adapter: eframe::wgpu::Limits) -> eframe::wgpu::Limits {
    eframe::wgpu::Limits { max_texture_dimension_2d: adapter.max_texture_dimension_2d.min(16384), ..adapter }
}

/// The window couldn't open (no usable graphics adapter or device): say so instead of quitting
/// silently, since release builds on Windows have no console. `gl_next`: the launch guard has
/// the next launch try OpenGL.
fn startup_failed(e: &eframe::Error, gl_next: bool) {
    let next = if gl_next { "The next launch will try OpenGL instead. " } else { "" };
    let message = format!(
        "Tokeffects couldn't start its window: {e}\n\n{next}Updating the graphics driver often helps. If it keeps happening, please report it at https://github.com/storytold/effectcraft/issues with this message."
    );
    // Standard error and the session log; standard error alone when RUST_LOG turned errors off.
    if log::log_enabled!(log::Level::Error) {
        log::error!("{message}");
    } else {
        eprintln!("Tokeffects: {message}");
    }
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Tokeffects")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

/// Session setup is prepared in a worker so loading media or plugins does not block the splash.
struct PreparedStartup {
    session: effectcraft_engine::Session,
    show_home: bool,
}

/// The shell is displayed as soon as the graphics context is ready, then swaps to the editor.
struct Desktop {
    app: Option<EffectcraftApp>,
    #[cfg(target_os = "macos")]
    menu: native_menu::NativeBar,
    #[cfg(target_os = "macos")]
    apple_events: apple_events::Opened,
    startup_rx: Option<std::sync::mpsc::Receiver<PreparedStartup>>,
    startup_error: Option<String>,
    gpu_failures: effectcraft_ui_egui::gpu_failure::GpuFailureBridge,
    notice: Option<String>,
    control_port: Option<u16>,
    logo: Option<egui::TextureHandle>,
    window: window::Tracker,
    launch: launch_guard::Launch,
    drawn: bool,
}

impl Desktop {
    fn poll_startup(&mut self, ctx: &egui::Context) {
        let received = match self.startup_rx.as_ref() {
            Some(rx) => rx.try_recv(),
            None => return,
        };
        match received {
            Ok(prepared) => {
                self.startup_rx = None;
                self.finish_startup(prepared, ctx);
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.startup_rx = None;
                self.startup_error = Some("Workspace loading failed. Restart Tokeffects and check the session log.".into());
                log::error!("Tokeffects startup worker stopped before it finished preparing the workspace");
            }
        }
    }

    fn finish_startup(&mut self, prepared: PreparedStartup, ctx: &egui::Context) {
        let mut app = EffectcraftApp::new(prepared.session);
        app.set_gpu_failure_bridge(self.gpu_failures.clone());
        app.ui.start_screen = prepared.show_home;
        if let Some(message) = self.notice.take() {
            app.ui.status = message;
        }

        if self.control_port.is_none()
            && let Some(recovery) = app.session.begin_recovery()
        {
            app.offer_recovery(recovery);
        }
        #[cfg(target_os = "linux")]
        if let Some(note) = wayland_drop_notice(
            std::env::var_os("WAYLAND_DISPLAY").as_deref(),
            std::env::var_os("WINIT_UNIX_BACKEND").as_deref(),
        ) && app.ui.status.is_empty()
        {
            app.ui.status = note.into();
        }

        app.hooks.pick_files = Some(Box::new(|exts: &[&str]| {
            rfd::FileDialog::new()
                .add_filter("Media", exts)
                .pick_files()
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        }));
        app.hooks.pick_save = Some(Box::new(|name: &str| {
            rfd::FileDialog::new()
                .add_filter("Tokeffects Project", &["ecproj"])
                .set_file_name(name)
                .save_file()
                .map(|p| p.to_string_lossy().to_string())
        }));
        if self.control_port.is_none() {
            app.hooks.device_lost = Some(Box::new(|saved: Option<Result<String, String>>| {
                let saved = match saved {
                    Some(Ok(path)) => format!("\n\nYour unsaved changes were saved to:\n{path}\n\nReopen Tokeffects to recover them."),
                    Some(Err(error)) => format!("\n\nYour unsaved changes could not be saved ({error})."),
                    None => String::new(),
                };
                rfd::MessageDialog::new()
                    .set_level(rfd::MessageLevel::Error)
                    .set_title("Tokeffects: graphics device lost")
                    .set_description(format!("The graphics driver stopped responding, so Tokeffects has to close.{saved}"))
                    .show();
            }));
        }
        app.hooks.pick_open_project = Some(Box::new(|| {
            rfd::FileDialog::new()
                .add_filter("Tokeffects Project", &["ecproj", "ecprojx"])
                .pick_file()
                .map(|p| p.to_string_lossy().to_string())
        }));
        app.hooks.system_theme = appearance::service();
        app.hooks.audio_device = Some(Box::new(audio_out::open));
        app.hooks.audio_devices = Some(Box::new(audio_out::devices));
        app.hooks.pick_folder = Some(Box::new(|| {
            rfd::FileDialog::new().pick_folder().map(|p| p.to_string_lossy().to_string())
        }));
        app.hooks.pick_save_file = Some(Box::new(|name: &str, ext: &str| {
            let path = std::path::Path::new(name);
            let mut dialog = if ext.is_empty() {
                rfd::FileDialog::new()
            } else {
                rfd::FileDialog::new().add_filter(ext, &[ext])
            };
            if let Some(dir) = path.parent().filter(|dir| dir.is_dir()) {
                dialog = dialog.set_directory(dir);
            }
            let file = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            dialog.set_file_name(file).save_file().map(|p| p.to_string_lossy().to_string())
        }));
        install_file_dialogs(&mut app, ctx);
        #[cfg(target_os = "macos")]
        {
            app.hooks.app_action = Some(Box::new(native_menu::app_action));
            app.hooks.clipboard_text = Some(Box::new(native_menu::clipboard_text));
        }
        if let Some(port) = self.control_port {
            disable_app_nap();
            let rx = control_server::start(port, ctx.clone());
            app = app.with_control(rx);
        }
        self.app = Some(app);
        log::info!("Tokeffects workspace ready");
    }

    fn splash(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        if self.logo.is_none() {
            match image::load_from_memory(ICON_PNG) {
                Ok(image) => {
                    let rgba = image.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    let pixels = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                    self.logo = Some(ctx.load_texture("tokeffects-splash-logo", pixels, egui::TextureOptions::LINEAR));
                }
                Err(error) => log::error!("could not load Tokeffects splash image: {error}"),
            }
        }
        let rect = ui.max_rect();
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
        let center = rect.center();
        if let Some(logo) = self.logo.as_ref() {
            let image_rect = egui::Rect::from_center_size(
                egui::pos2(center.x, center.y - 48.0),
                egui::vec2(92.0, 92.0),
            );
            painter.image(
                logo.id(),
                image_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        let (title, subtitle) = if let Some(message) = self.startup_error.as_deref() {
            ("Unable to start Tokeffects", message)
        } else {
            ("Tokeffects", "Preparing your workspace")
        };
        painter.text(
            egui::pos2(center.x, center.y + 22.0),
            egui::Align2::CENTER_CENTER,
            title,
            egui::FontId::new(22.0, egui::FontFamily::Proportional),
            egui::Color32::from_gray(232),
        );
        painter.text(
            egui::pos2(center.x, center.y + 49.0),
            egui::Align2::CENTER_CENTER,
            subtitle,
            egui::FontId::new(11.5, egui::FontFamily::Proportional),
            egui::Color32::from_gray(136),
        );
        if self.startup_error.is_none() {
            let bar = egui::Rect::from_center_size(egui::pos2(center.x, center.y + 75.0), egui::vec2(92.0, 3.0));
            painter.rect_filled(bar, 1.5, egui::Color32::from_gray(38));
            let t = ui.input(|input| input.time);
            let offset = -36.0 + (t * 42.0).rem_euclid(72.0);
            let marker = egui::Rect::from_center_size(
                egui::pos2(center.x + offset, bar.center().y),
                egui::vec2(20.0, 3.0),
            );
            painter.rect_filled(marker, 1.5, egui::Color32::from_gray(194));
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }
    }
}

impl eframe::App for Desktop {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(app) = self.app.as_mut() {
            #[cfg(target_os = "macos")]
            self.menu.update(app, ctx);
            app.logic(ctx, frame);
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.poll_startup(ui.ctx());
        if let Some(app) = self.app.as_mut() {
            app.ui(ui, frame);
            self.window.frame(ui.ctx(), app.session.config.as_deref());
            if !self.drawn {
                self.drawn = true;
                // Start the optional system-font inventory only after the first editor frame is
                // visible, so a large font collection cannot compete with GPU/window startup.
                effectcraft_engine::text::fonts::scan_system_in_background();
                log::info!("first editor frame drawn");
            }
        } else {
            self.splash(ui);
        }
        if let Some(left) = self.launch.frame_drawn() {
            ui.ctx().request_repaint_after(left);
        }
    }

    fn on_exit(&mut self) {
        self.launch.exited();
        if let Some(app) = self.app.as_mut() {
            self.window.save(app.session.config.as_deref());
            app.on_exit();
        }
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        let ready = self.app.as_ref().is_some_and(|app| app.is_ready());
        #[cfg(target_os = "macos")]
        self.apple_events.feed(ready, ctx, raw_input);
        if let Some(app) = self.app.as_mut() {
            app.raw_input_hook(ctx, raw_input);
        }
        let _ = ready;
    }
}

/// Driven by an agent (`--control`): don't activate the app on launch, so the user's keyboard
/// focus stays where it is (macOS; winit activates ignoring other apps by default).
fn agent_event_loop(agent: bool) -> Option<eframe::EventLoopBuilderHook> {
    #[cfg(target_os = "macos")]
    if agent {
        return Some(Box::new(|b| {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            b.with_activate_ignoring_other_apps(false);
        }));
    }
    let _ = agent;
    None
}

/// Agents drive the app while its window is covered or on another Space. macOS App Nap would
/// throttle the event loop then, stalling the control channel; opt out for `--control` runs.
fn disable_app_nap() {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::{NSActivityOptions, NSProcessInfo, NSString};
        let info = NSProcessInfo::processInfo();
        let reason = NSString::from_str("Tokeffects control channel");
        let opts = NSActivityOptions::UserInitiatedAllowingIdleSystemSleep | NSActivityOptions::LatencyCritical;
        // Leaked on purpose: the activity lasts for the life of the process.
        std::mem::forget(info.beginActivityWithOptions_reason(opts, &reason));
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, parse_args};
    use std::env::VarError;

    fn parse_env(args: &[&str], env_port: Result<String, VarError>) -> Result<Option<Args>, String> {
        parse_args(args.iter().map(|a| a.to_string()), env_port)
    }

    fn parse(args: &[&str]) -> Option<Args> {
        parse_env(args, Err(VarError::NotPresent)).unwrap()
    }

    fn fails(args: &[&str]) -> String {
        parse_env(args, Err(VarError::NotPresent)).unwrap_err()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn wayland_drop_notice_recommends_import_or_xwayland() {
        use std::ffi::OsStr;
        let warning = super::wayland_drop_notice(Some(OsStr::new("wayland-0")), None).unwrap();
        assert!(warning.contains("File ▸ Import") && warning.contains("XWayland"));
        assert!(super::wayland_drop_notice(None, None).is_none());
        assert!(super::wayland_drop_notice(Some(OsStr::new("")), None).is_none());
        assert!(super::wayland_drop_notice(Some(OsStr::new("wayland-0")), Some(OsStr::new("x11"))).is_none());
    }

    /// The window's wgpu instance skips indirect-call validation (Intel Metal, #1153).
    #[test]
    fn the_window_instance_skips_indirect_call_validation() {
        if std::env::var_os("WGPU_VALIDATION_INDIRECT_CALL").is_some() {
            return;
        }
        let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = super::wgpu_options(None).wgpu_setup else {
            panic!("the default setup creates its own instance");
        };
        // Intel Metal can't compile the validation's compute shader, and the device is lost.
        assert!(!setup.instance_descriptor.flags.contains(eframe::wgpu::InstanceFlags::VALIDATION_INDIRECT_CALL));
    }

    /// The window's device never asks for more than the adapter offers (#198).
    #[test]
    fn device_limits_fit_the_adapter() {
        use eframe::wgpu::Limits;
        for adapter in [Limits::downlevel_defaults(), Limits::downlevel_webgl2_defaults(), Limits::default()] {
            let l = super::device_limits(adapter.clone());
            assert!(l.check_limits(&adapter), "{l:?}");
            assert_eq!(l.max_storage_buffers_per_shader_stage, adapter.max_storage_buffers_per_shader_stage);
        }
        let big = Limits { max_texture_dimension_2d: 32768, ..Limits::default() };
        assert_eq!(super::device_limits(big).max_texture_dimension_2d, 16384);
    }

    /// A positional .ecprojx is a project file, not an unsupported media import (#528).
    #[test]
    fn launch_classifies_xml_and_json_projects_as_projects() {
        for path in ["Copy.ecprojx", "/tmp/Scene.ecproj", "C:\\\\work\\\\Shot.ECPROJX"] {
            assert!(super::is_project_file(path), "{path}");
        }
        for path in ["scene.ecprojx.png", "scene.mp4", "scene", "copy.ecprojx.bak"] {
            assert!(!super::is_project_file(path), "{path}");
        }
        let files = ["scene.ecprojx".to_string(), "clip.mov".to_string()];
        assert_eq!(files.iter().find(|f| super::is_project_file(f)).map(String::as_str), Some("scene.ecprojx"));
        assert_eq!(files.iter().filter(|f| !super::is_project_file(f)).map(String::as_str).collect::<Vec<_>>(), vec!["clip.mov"],);
    }

    /// Launching without a project opens an empty project, not the demo (#204).
    #[test]
    fn no_arguments_start_an_empty_project() {
        assert_eq!(parse(&[]), Some(Args::default()));
        assert!(parse(&["--demo"]).is_some_and(|a| a.demo));
        assert!(parse(&["--demo", "--empty"]).is_some_and(|a| !a.demo));
        let a = parse(&["--control", "9877", "--home", "a.ecproj"]).unwrap_or_default();
        assert_eq!((a.control_port, a.home, a.files), (Some(9877), Some(true), vec!["a.ecproj".to_string()]));
        assert_eq!(parse(&["--version"]), None);
        assert_eq!(parse_env(&[], Ok("9000".into())).unwrap().map(|a| a.control_port), Some(Some(9000)));
    }

    /// A missing, non-numeric or zero `--control` value stops the launch instead of starting
    /// without a control server, and the argument after `--control` isn't reinterpreted (#997).
    #[test]
    fn control_needs_a_port_from_1_to_65535() {
        let need = "--control needs a port number from 1 to 65535";
        assert_eq!(fails(&["--control"]), need);
        assert_eq!(fails(&["--control", "--demo"]), format!("{need}, not `--demo`"));
        assert_eq!(fails(&["--control", "--version"]), format!("{need}, not `--version`"));
        for bad in ["abc", "0", "70000", "-1", ""] {
            assert_eq!(fails(&["--control", bad]), format!("{need}, not `{bad}`"));
            assert_eq!(fails(&[&format!("--control={bad}")]), format!("{need}, not `{bad}`"));
        }
        assert_eq!(parse(&["--control=9877"]).map(|a| a.control_port), Some(Some(9877)));
        assert_eq!(parse(&["--control", "65535"]).map(|a| a.control_port), Some(Some(65535)));
    }

    /// An unknown option is an error, not a file to import (#997); `--` ends the options.
    #[test]
    fn unknown_options_are_errors_not_files() {
        for bad in ["--automation-read-root", "--automation-write-root=/tmp", "-h", "--Demo", "--demo=yes"] {
            assert_eq!(fails(&[bad, "/tmp/folder"]), format!("unknown option `{bad}`"));
        }
        // Parsed left to right: an error before `--version` wins.
        assert_eq!(fails(&["--nope", "--version"]), "unknown option `--nope`");
        // Finder's process serial number is ignored, not imported or refused.
        assert_eq!(parse(&["-psn_0_12345"]).map(|a| a.files), Some(vec![]));
        let a = parse(&["clip.mov", "-", "--", "--demo", "-name.png"]).unwrap_or_default();
        assert_eq!(a.files, ["clip.mov", "-", "--demo", "-name.png"]);
        assert!(!a.demo);
    }

    /// `--control` wins over `TOKCRAFT_CONTROL_PORT`; the variable is checked only when it is
    /// used, so a bad value doesn't stop `--control` or `--version`.
    #[test]
    fn control_port_variable_is_checked_when_used() {
        let need = "TOKCRAFT_CONTROL_PORT needs a port number from 1 to 65535";
        assert_eq!(parse_env(&[], Ok("abc".into())).unwrap_err(), format!("{need}, not `abc`"));
        assert_eq!(parse_env(&[], Ok("0".into())).unwrap_err(), format!("{need}, not `0`"));
        assert_eq!(parse_env(&[], Ok(String::new())), Ok(Some(Args::default())));
        assert_eq!(parse_env(&["--control", "9877"], Ok("abc".into())).unwrap().map(|a| a.control_port), Some(Some(9877)));
        assert_eq!(parse_env(&["--version"], Ok("abc".into())), Ok(None));
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let not_utf8 = VarError::NotUnicode(std::ffi::OsString::from_vec(vec![0x39, 0xff]));
            assert_eq!(parse_env(&[], Err(not_utf8)).unwrap_err(), format!("{need}, not `9\u{fffd}`"));
        }
    }

    /// #1275: a blocking desktop-portal dialog on Linux leaves the window not answering the
    /// compositor while it is open, so the commands' file dialogs go through the non-blocking
    /// host hook there (as they do on macOS, #511). Windows' dialog pumps its own messages.
    #[test]
    fn commands_file_dialogs_do_not_block_the_frame() {
        let mut app = effectcraft_ui_egui::EffectcraftApp::new(effectcraft_engine::Session::default());
        super::install_file_dialogs(&mut app, &egui::Context::default());
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        assert!(app.hooks.file_dialog_async.is_some(), "the commands' file dialogs must not block the frame");
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        assert!(app.hooks.file_dialog_async.is_none(), "the blocking dialog pumps its own messages here");
    }
}
