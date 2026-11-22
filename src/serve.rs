//! One poller, many local listeners. ESPN still sees HTTP ≥15s apart.
//!
//!   GET /health   JSON liveness
//!   GET /clock    JSON snapshot {league: [Game]}
//!   GET /events   SSE: event clock on fingerprint change only

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use espnclock::{
    fetch_scoreboard, fingerprint, league_is_hot, next_idle_or_hot_secs, Game, LEAGUES,
    MIN_FETCH_SECS,
};

const MAX_CLIENTS: usize = 16;

#[derive(Clone, serde::Serialize)]
struct Snapshot {
    fetches: u64,
    leagues_hot: usize,
    boards: HashMap<String, Vec<Game>>,
}

struct Client {
    stream: TcpStream,
}

struct Hub {
    snapshot: Mutex<Snapshot>,
    clients: Mutex<Vec<Client>>,
}

pub fn serve(bind: &str, min_fetch: u64) -> Result<(), String> {
    let min_fetch = min_fetch.max(MIN_FETCH_SECS);
    let listener = TcpListener::bind(bind).map_err(|e| e.to_string())?;
    eprintln!("espnclock serve http://{bind}  (espn ≥{min_fetch}s, localhost fan-out)");
    let hub = Arc::new(Hub {
        snapshot: Mutex::new(Snapshot {
            fetches: 0,
            leagues_hot: 0,
            boards: HashMap::new(),
        }),
        clients: Mutex::new(Vec::new()),
    });
    {
        let hub = Arc::clone(&hub);
        std::thread::spawn(move || poller(hub, min_fetch));
    }
    {
        let hub = Arc::clone(&hub);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(15));
            keepalive(&hub);
        });
    }
    for conn in listener.incoming() {
        let Ok(stream) = conn else { continue };
        let hub = Arc::clone(&hub);
        std::thread::spawn(move || handle(stream, hub));
    }
    Ok(())
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn poller(hub: Arc<Hub>, min_fetch: u64) {
    let mut due: HashMap<&str, i64> = LEAGUES.iter().map(|l| (l.slug, 0i64)).collect();
    let mut prints: HashMap<&str, String> = HashMap::new();
    let mut i = 0usize;
    loop {
        let now = now_unix();
        let n = LEAGUES.len();
        let mut did = false;
        for _ in 0..n {
            let lg = &LEAGUES[i % n];
            i += 1;
            if due.get(lg.slug).copied().unwrap_or(0) > now {
                continue;
            }
            match fetch_scoreboard(lg, None) {
                Ok(games) => {
                    let hot = league_is_hot(&games, now);
                    due.insert(lg.slug, now + next_idle_or_hot_secs(hot) as i64);
                    let fp = fingerprint(&games);
                    let changed = prints.get(lg.slug).map(String::as_str) != Some(fp.as_str());
                    prints.insert(lg.slug, fp);
                    let mut snap = hub.snapshot.lock().expect("snap");
                    snap.fetches += 1;
                    snap.boards.insert(lg.slug.to_string(), games);
                    snap.leagues_hot = snap
                        .boards
                        .values()
                        .filter(|g| league_is_hot(g, now))
                        .count();
                    let payload = serde_json::to_string(&*snap).unwrap_or_else(|_| "{}".into());
                    drop(snap);
                    if changed {
                        broadcast(&hub, "clock", &payload);
                    }
                }
                Err(e) => {
                    eprintln!("espnclock poll {0}: {e}", lg.slug);
                    due.insert(lg.slug, now + next_idle_or_hot_secs(false) as i64);
                }
            }
            did = true;
            break;
        }
        if did {
            std::thread::sleep(Duration::from_secs(min_fetch));
        } else {
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

fn broadcast(hub: &Hub, event: &str, data: &str) {
    let frame = format!("event: {event}\ndata: {data}\n\n");
    let mut clients = hub.clients.lock().expect("clients");
    clients.retain_mut(|c| c.stream.write_all(frame.as_bytes()).is_ok());
}

fn keepalive(hub: &Hub) {
    let mut clients = hub.clients.lock().expect("clients");
    clients.retain_mut(|c| c.stream.write_all(b": keepalive\n\n").is_ok());
}

fn handle(mut stream: TcpStream, hub: Arc<Hub>) {
    let mut buf = [0u8; 1024];
    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return,
    };
    let req = String::from_utf8_lossy(&buf[..n]);
    let path = req
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");
    if path.starts_with("/events") {
        if !register(&hub, &stream) {
            let _ =
                stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");
            return;
        }
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n",
        );
        let snap = hub.snapshot.lock().expect("snap");
        if let Ok(js) = serde_json::to_string(&*snap) {
            let _ = stream.write_all(format!("event: clock\ndata: {js}\n\n").as_bytes());
        }
        // ownership moved into hub; this thread ends, poller writes
        return;
    }
    let (status, body, ctype) = if path.starts_with("/health") {
        let snap = hub.snapshot.lock().expect("snap");
        (
            "200 OK",
            format!(
                "{{\"ok\":true,\"fetches\":{},\"leagues_hot\":{},\"clients\":{}}}",
                snap.fetches,
                snap.leagues_hot,
                hub.clients.lock().map(|c| c.len()).unwrap_or(0)
            ),
            "application/json",
        )
    } else if path.starts_with("/clock") {
        let snap = hub.snapshot.lock().expect("snap");
        (
            "200 OK",
            serde_json::to_string(&*snap).unwrap_or_else(|_| "{}".into()),
            "application/json",
        )
    } else {
        ("404 Not Found", "not found".into(), "text/plain")
    };
    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn register(hub: &Hub, stream: &TcpStream) -> bool {
    let Ok(clone) = stream.try_clone() else {
        return false;
    };
    let mut clients = hub.clients.lock().expect("clients");
    if clients.len() >= MAX_CLIENTS {
        return false;
    }
    let _ = clone.set_nodelay(true);
    clients.push(Client { stream: clone });
    true
}
