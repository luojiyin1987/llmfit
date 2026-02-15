mod hardware;
mod models;
mod fit;
mod display;
mod tui_app;
mod tui_ui;
mod tui_events;

use clap::{Parser, Subcommand};
use hardware::SystemSpecs;
use models::ModelDatabase;
use fit::ModelFit;

#[derive(Parser)]
#[command(name = "llmfit")]
#[command(about = "Right-size LLM models to your system's hardware", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Show only models that perfectly match recommended specs
    #[arg(short, long)]
    perfect: bool,

    /// Limit number of results
    #[arg(short = 'n', long)]
    limit: Option<usize>,

    /// Use classic CLI table output instead of TUI
    #[arg(long)]
    cli: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show system hardware specifications
    System,

    /// List all available LLM models
    List,

    /// Find models that fit your system (classic table output)
    Fit {
        /// Show only models that perfectly match recommended specs
        #[arg(short, long)]
        perfect: bool,

        /// Limit number of results
        #[arg(short = 'n', long)]
        limit: Option<usize>,
    },

    /// Search for specific models
    Search {
        /// Search query (model name, provider, or size)
        query: String,
    },

    /// Show detailed information about a specific model
    Info {
        /// Model name or partial name to look up
        model: String,
    },
}

fn run_fit(perfect: bool, limit: Option<usize>) {
    let specs = SystemSpecs::detect();
    let db = ModelDatabase::new();

    specs.display();

    let mut fits: Vec<ModelFit> = db
        .get_all_models()
        .iter()
        .map(|m| ModelFit::analyze(m, &specs))
        .collect();

    if perfect {
        fits.retain(|f| f.fit_level == fit::FitLevel::Perfect);
    }

    fits = fit::rank_models_by_fit(fits);

    if let Some(n) = limit {
        fits.truncate(n);
    }

    display::display_model_fits(&fits);
}

fn run_tui() -> std::io::Result<()> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Create app state
    let mut app = tui_app::App::new();

    // Main loop
    loop {
        terminal.draw(|frame| {
            tui_ui::draw(frame, &mut app);
        })?;

        tui_events::handle_events(&mut app)?;

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // If a subcommand is given, use classic CLI mode
    if let Some(command) = cli.command {
        match command {
            Commands::System => {
                let specs = SystemSpecs::detect();
                specs.display();
            }

            Commands::List => {
                let db = ModelDatabase::new();
                display::display_all_models(db.get_all_models());
            }

            Commands::Fit { perfect, limit } => {
                run_fit(perfect, limit);
            }

            Commands::Search { query } => {
                let db = ModelDatabase::new();
                let results = db.find_model(&query);
                display::display_search_results(&results, &query);
            }

            Commands::Info { model } => {
                let db = ModelDatabase::new();
                let specs = SystemSpecs::detect();
                let results = db.find_model(&model);

                if results.is_empty() {
                    println!("\nNo model found matching '{}'", model);
                    return;
                }

                if results.len() > 1 {
                    println!("\nMultiple models found. Please be more specific:");
                    for m in results {
                        println!("  - {}", m.name);
                    }
                    return;
                }

                let fit = ModelFit::analyze(results[0], &specs);
                display::display_model_detail(&fit);
            }
        }
        return;
    }

    // If --cli flag, use classic fit output
    if cli.cli {
        run_fit(cli.perfect, cli.limit);
        return;
    }

    // Default: launch TUI
    if let Err(e) = run_tui() {
        eprintln!("Error running TUI: {}", e);
        std::process::exit(1);
    }
}
