//! Typed ESPN site.v2 scoreboard clock. Unofficial; no affiliation with ESPN.

use serde::Deserialize;
use serde_json::Value;

pub const USER_AGENT: &str = "espnclock/0.1.0 (+https://github.com/copyleftdev/espnclock)";
pub const SITE_V2: &str = "https://site.api.espn.com/apis/site/v2/sports";

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
