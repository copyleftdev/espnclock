//! Typed ESPN site.v2 scoreboard clock. Unofficial; no affiliation with ESPN.

use serde::Deserialize;
use serde_json::Value;

pub const USER_AGENT: &str = "espnclock/0.2.0 (+https://github.com/copyleftdev/espnclock)";
pub const SITE_V2: &str = "https://site.api.espn.com/apis/site/v2/sports";
/// Floor between ESPN fetches (any league). Above scoreboard max-age (~2s).
pub const MIN_FETCH_SECS: u64 = 15;
/// Quiet league recheck.
pub const IDLE_SECS: u64 = 300;
/// `pre` games this close to commence are "hot".
pub const HOT_PRE_SECS: i64 = 7_200;

#[derive(Debug, Clone, Copy)]
pub struct League {
    pub slug: &'static str,
    pub sport: &'static str,
    pub league: &'static str,
}

pub const LEAGUES: &[League] = &[
    League {
        slug: "nfl",
        sport: "football",
        league: "nfl",
    },
    League {
        slug: "ncaaf",
        sport: "football",
        league: "college-football",
    },
    League {
        slug: "nba",
        sport: "basketball",
        league: "nba",
    },
    League {
        slug: "wnba",
        sport: "basketball",
        league: "wnba",
    },
    League {
        slug: "ncaab",
        sport: "basketball",
        league: "mens-college-basketball",
    },
    League {
        slug: "mlb",
        sport: "baseball",
        league: "mlb",
    },
    League {
        slug: "nhl",
        sport: "hockey",
        league: "nhl",
    },
    League {
        slug: "ufc",
        sport: "mma",
        league: "ufc",
    },
];

pub fn league(slug: &str) -> Option<&'static League> {
    LEAGUES.iter().find(|l| l.slug.eq_ignore_ascii_case(slug))
}

pub fn scoreboard_url(lg: &League, dates: Option<&str>) -> String {
    let mut u = format!("{}/{}/{}/scoreboard", SITE_V2, lg.sport, lg.league);
    if let Some(d) = dates {
        u.push_str("?dates=");
        u.push_str(d);
    }
    u
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Game {
    pub id: String,
    pub commence: String,
    pub state: String,
    pub detail: String,
    pub away: String,
    pub home: String,
    pub away_score: String,
    pub home_score: String,
    pub short_name: String,
}

impl Game {
    pub fn started(&self) -> bool {
        self.state != "pre"
    }

    pub fn matches(&self, a: &str, b: &str) -> bool {
        let a = a.to_ascii_uppercase();
        let b = b.to_ascii_uppercase();
        (self.away == a && self.home == b) || (self.away == b && self.home == a)
    }

    /// In-progress, or pre within HOT_PRE_SECS of commence. Finals are cold.
    pub fn is_hot(&self, now_unix: i64) -> bool {
        if self.state == "in" {
            return true;
        }
        if self.state != "pre" {
            return false;
        }
        match commence_unix(&self.commence) {
            Some(t) => t - now_unix <= HOT_PRE_SECS && now_unix - t < 3_600,
            None => false,
        }
    }
}

/// Pad `2026-11-22T21:25Z` to RFC3339 seconds.
pub fn commence_unix(iso: &str) -> Option<i64> {
    let padded = if iso.ends_with('Z') && iso.chars().filter(|c| *c == ':').count() == 1 {
        format!("{}:00Z", iso.trim_end_matches('Z'))
    } else {
        iso.to_string()
    };
    time::OffsetDateTime::parse(&padded, &time::format_description::well_known::Rfc3339)
        .ok()
        .map(|t| t.unix_timestamp())
}

pub fn league_is_hot(games: &[Game], now_unix: i64) -> bool {
    games.iter().any(|g| g.is_hot(now_unix))
}

pub fn fingerprint(games: &[Game]) -> String {
    let mut parts: Vec<String> = games
        .iter()
        .map(|g| format!("{}:{}:{}", g.id, g.state, g.detail))
        .collect();
    parts.sort();
    parts.join("|")
}

pub fn next_idle_or_hot_secs(hot: bool) -> u64 {
    if hot {
        MIN_FETCH_SECS
    } else {
        IDLE_SECS
    }
}

/// Default serve bind is loopback. Non-loopback needs an explicit expose flag.
pub fn bind_is_loopback(bind: &str) -> bool {
    let host = bind.rsplit_once(':').map(|(h, _)| h).unwrap_or(bind);
    host == "127.0.0.1" || host == "localhost" || host == "[::1]" || host == "::1"
}

#[derive(Deserialize)]
struct Board {
    events: Option<Vec<Value>>,
}

pub fn parse_scoreboard(v: &Value) -> Result<Vec<Game>, String> {
    let board: Board =
        serde_json::from_value(v.clone()).map_err(|e| format!("scoreboard shape: {e}"))?;
    let mut out = Vec::new();
    for ev in board.events.unwrap_or_default() {
        if let Some(g) = parse_event(&ev) {
            out.push(g);
        }
    }
    Ok(out)
}

fn parse_event(ev: &Value) -> Option<Game> {
    let id = ev.get("id")?.as_str()?.to_string();
    let commence = ev.get("date")?.as_str()?.to_string();
    let short_name = ev
        .get("shortName")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let st = ev.get("status")?.get("type")?;
    let state = st.get("state")?.as_str()?.to_string();
    let detail = st
        .get("shortDetail")
        .or_else(|| st.get("detail"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let comps = ev.get("competitions")?.as_array()?;
    let competitors = comps.first()?.get("competitors")?.as_array()?;
    let mut away = String::new();
    let mut home = String::new();
    let mut away_score = String::new();
    let mut home_score = String::new();
    for c in competitors {
        let abbr = c
            .get("team")?
            .get("abbreviation")?
            .as_str()?
            .to_ascii_uppercase();
        let score = c
            .get("score")
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string();
        match c.get("homeAway").and_then(Value::as_str) {
            Some("home") => {
                home = abbr;
                home_score = score;
            }
            Some("away") => {
                away = abbr;
                away_score = score;
            }
            _ => {}
        }
    }
    if away.is_empty() || home.is_empty() {
        return None;
    }
    Some(Game {
        id,
        commence,
        state,
        detail,
        away,
        home,
        away_score,
        home_score,
        short_name,
    })
}

pub fn fetch_scoreboard(lg: &League, dates: Option<&str>) -> Result<Vec<Game>, String> {
    let url = scoreboard_url(lg, dates);
    let body: Value = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .timeout(std::time::Duration::from_secs(12))
        .call()
        .map_err(|e| format!("espn fetch: {e}"))?
        .into_json()
        .map_err(|e| format!("espn json: {e}"))?;
    parse_scoreboard(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fixture_pre_and_in() {
        let raw = include_str!("../tests/fixtures/nfl_scoreboard.json");
        let v: Value = serde_json::from_str(raw).unwrap();
        let games = parse_scoreboard(&v).unwrap();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].state, "in");
        assert!(games[0].started());
        assert_eq!(games[0].away, "MIA");
        assert_eq!(games[0].home, "BUF");
        assert_eq!(games[1].state, "pre");
        assert!(!games[1].started());
        assert!(games[1].matches("LV", "DEN"));
        assert!(games[1].matches("den", "lv"));
        let t = commence_unix("2026-11-22T21:25Z").unwrap();
        assert_eq!(t, commence_unix("2026-11-22T21:25:00Z").unwrap());
        assert!(games[1].is_hot(t - 3_600));
        assert!(!games[1].is_hot(t - 8_000));
        assert!(games[0].is_hot(t));
        assert!(!league_is_hot(&games[1..], t + 10_000));
        assert_eq!(next_idle_or_hot_secs(true), 15);
        assert_eq!(next_idle_or_hot_secs(false), 300);
        assert!(bind_is_loopback("127.0.0.1:8796"));
        assert!(bind_is_loopback("localhost:8796"));
        assert!(!bind_is_loopback("0.0.0.0:8796"));
    }

    #[test]
    fn league_table() {
        assert_eq!(league("NFL").unwrap().sport, "football");
        assert_eq!(league("ufc").unwrap().league, "ufc");
        assert!(league("kxnfl").is_none());
        assert!(
            scoreboard_url(league("nfl").unwrap(), Some("20261122")).ends_with("dates=20261122")
        );
    }
}
