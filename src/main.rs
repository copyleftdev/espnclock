use clap::{Parser, Subcommand};
use espnclock::{fetch_scoreboard, league, Game, LEAGUES, MIN_FETCH_SECS};

mod serve;

#[derive(Parser)]
#[command(
    name = "espnclock",
    version,
    about = "Typed event clock over ESPN site.v2 scoreboard JSON (unofficial)"
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    date: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// List supported league slugs
    Leagues,
    /// Today's (or --date) scoreboard
    Board { league: String },
    /// Only games that have left pre (in/post)
    Started { league: String },
    /// One matchup: exit 0 if found, 3 if missing
    Status {
        league: String,
        away: String,
        home: String,
    },
    /// One ESPN poller, local SSE fan-out (does not multiply ESPN hits)
    Serve {
        #[arg(long, default_value = "127.0.0.1:8796")]
        bind: String,
        /// Floor seconds between ESPN fetches (clamped ≥15)
        #[arg(long, default_value_t = MIN_FETCH_SECS)]
        interval: u64,
    },
}

fn print_games(games: &[Game], json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(games).unwrap());
        return;
    }
    for g in games {
        println!(
            "{:<4} {:<4} @ {:<4}  {:<4}  {}  {}",
            g.state, g.away, g.home, g.detail, g.commence, g.id
        );
    }
}

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    match cli.cmd {
        Cmd::Leagues => {
            if cli.json {
                let slugs: Vec<&str> = LEAGUES.iter().map(|l| l.slug).collect();
                println!("{}", serde_json::to_string_pretty(&slugs).unwrap());
            } else {
                for l in LEAGUES {
                    println!("{:<6} {}/{}", l.slug, l.sport, l.league);
                }
            }
            0
        }
        Cmd::Board { league: slug } => match league(&slug) {
            None => {
                eprintln!("unknown league {slug}");
                1
            }
            Some(lg) => match fetch_scoreboard(lg, cli.date.as_deref()) {
                Ok(games) => {
                    print_games(&games, cli.json);
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    2
                }
            },
        },
        Cmd::Started { league: slug } => match league(&slug) {
            None => {
                eprintln!("unknown league {slug}");
                1
            }
            Some(lg) => match fetch_scoreboard(lg, cli.date.as_deref()) {
                Ok(games) => {
                    let started: Vec<Game> = games.into_iter().filter(|g| g.started()).collect();
                    print_games(&started, cli.json);
                    0
                }
                Err(e) => {
                    eprintln!("{e}");
                    2
                }
            },
        },
        Cmd::Status {
            league: slug,
            away,
            home,
        } => match league(&slug) {
            None => {
                eprintln!("unknown league {slug}");
                1
            }
            Some(lg) => match fetch_scoreboard(lg, cli.date.as_deref()) {
                Err(e) => {
                    eprintln!("{e}");
                    2
                }
                Ok(games) => match games.into_iter().find(|g| g.matches(&away, &home)) {
                    None => {
                        eprintln!("no matchup {away} {home}");
                        3
                    }
                    Some(g) => {
                        print_games(&[g], cli.json);
                        0
                    }
                },
            },
        },
        Cmd::Serve { bind, interval } => match serve::serve(&bind, interval) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("{e}");
                2
            }
        },
    }
}
