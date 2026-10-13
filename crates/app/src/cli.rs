//! 사용자 AI 에이전트용 CLI(D14 보강) — `peers` · `ask` · `wait` · `status` · `history`. `ask` 하나 = 앱의 CLI 트랜잭션 하나(#1c) — 앱이 주인이고
//! CLI 는 그 상태를 본다(중복 키 · 이어 받기 · 닿지 않음 멈춤). 떠 있는 주 인스턴스의 로컬 API 를
//! `instance.json` 의 bearer 토큰으로 부른다(웹UI 와 같은 신뢰 경계). 앱을 띄우지 않는다 — 실행 중이 아니면 종료 코드 3.
//! CLI 로 쓴 글은 `via = "cli"` 로 양쪽 이력에 남는다. 파이프로 부르는 것을 전제한다(릴리스는 콘솔 없는 서브시스템).
use crate::handoff::local_request;
use crate::server::CLI_API;
use an_core::instance::InstanceInfo;
use an_core::paths::Paths;
use serde_json::{Value, json};
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub const EXIT_OK: i32 = 0;
/// 사용법 · 인자 오류(모르는 명령 · 상대 없음 · 상대 · 에이전트 모호).
pub const EXIT_USAGE: i32 = 2;
/// coworkAgent 가 실행 중이 아니다(`instance.json` 없음 · 형식 오류 · 연결 거부).
pub const EXIT_NOT_RUNNING: i32 = 3;
/// 로컬 API 거부(401 토큰 · 4xx · 5xx — 페어링 안 됨 · 상대 거절 포함).
pub const EXIT_API: i32 = 4;
/// 답 실패(에이전트 실행 오류 · 상대에 못 보냄).
pub const EXIT_REPLY_FAILED: i32 = 5;
/// 답 대기 시간 초과(안전판 — 앱 판정이 먼저 끝낸다).
pub const EXIT_TIMEOUT: i32 = 6;
/// 상대에 정해진 시간 닿지 않아 멈춤(#1c) — 트랜잭션은 남는다, `wait <tx>` 로 이어 받는다.
pub const EXIT_STALLED: i32 = 7;

/// CLI 자체 대기 안전판(#1c) — 끝은 앱의 트랜잭션 판정(답 · 실패 · 멈춤)이 정한다. 상대가 대기 · 실행 중으로 확인되는 동안은 기다린다.
pub const WAIT_LIMIT: Duration = Duration::from_secs(24 * 3600);
/// 답 확인 간격.
const POLL: Duration = Duration::from_secs(1);
/// 로컬 API 요청 한 번의 상한 — 새 대화는 상대에 닿을 때까지 기다린다.
const REQUEST_DEADLINE: Duration = Duration::from_secs(60);
/// CLI 로 쓴 글의 발화자(웹UI `view.js` 와 같은 문구).
const CLI_LABEL: &str = "AI 에이전트(CLI)";

pub const USAGE: &str = "사용법:
  coworkAgent peers [--json]
  coworkAgent ask <상대> \"<질문>\" [--agent <이름>] [--conv <대화ID>] [--id <키>] [--no-wait] [--json]
  coworkAgent wait <트랜잭션> [--json]
  coworkAgent status <트랜잭션> [--json]
  coworkAgent history [<상대>] [--conv <대화ID>] [--json]
--id <키> 로 같은 질문을 다시 실행하면 새로 보내지 않고 그 트랜잭션의 결과를 받는다(1 ~ 64자 영문 · 숫자 · . _ -). 종료 코드 7 = 상대에 닿지 않아 멈춤(wait 로 이어 받기).
--conv 와 함께 준 --agent 가 그 대화 에이전트가 아니면 그 상대 PC 의 다른 에이전트를 부른다(멘션 — 내가 연 대화 · 자동 왕복 아님).
<상대> = 피어 표시 이름 또는 지문 앞자리(4자 이상). 본문이 -- 로 시작하면 앞에 -- 를 둔다(그 뒤는 전부 인자)";

/// 첫 인자가 CLI 하위 명령인가 — `main` 이 트레이 시작 전에 가른다.
pub fn is_command(arg: &str) -> bool {
    matches!(arg, "peers" | "ask" | "wait" | "status" | "history")
}

/// 실행 파일 진입점 — 표준 출력 · 오류에 쓰고 종료 코드를 돌려준다.
pub fn main(args: &[String]) -> i32 {
    let Some(root) = Paths::default_root() else {
        eprintln!("사용자 데이터 경로를 찾지 못했다");
        return EXIT_NOT_RUNNING;
    };
    let (mut out, mut err) = (io::stdout().lock(), io::stderr().lock());
    let code = run(
        args,
        &Paths::under(root).instance_file(),
        &mut out,
        &mut err,
        WAIT_LIMIT,
    );
    let _ = out.flush();
    let _ = err.flush();
    code
}

/// `args` = 하위 명령부터. 결과는 `out`, 실패 사유는 `err`, 종료 코드를 돌려준다.
pub fn run(
    args: &[String],
    instance_file: &Path,
    out: &mut dyn Write,
    err: &mut dyn Write,
    wait_limit: Duration,
) -> i32 {
    let result = Cli::parse(args).and_then(|cli| {
        let api = Api::open(instance_file)?;
        api.handshake()?;
        cli.exec(&api, out, err, wait_limit)
    });
    match result {
        Ok(()) => EXIT_OK,
        Err(Fail(code, msg)) => {
            let _ = writeln!(err, "{}", printable(&msg));
            code
        }
    }
}

/// 실패 — 종료 코드 + 사유.
#[derive(Debug)]
struct Fail(i32, String);

fn usage(msg: impl std::fmt::Display) -> Fail {
    Fail(EXIT_USAGE, format!("{msg}"))
}

fn not_running(why: impl std::fmt::Display) -> Fail {
    Fail(
        EXIT_NOT_RUNNING,
        format!("coworkAgent 가 실행 중이 아니다 — 트레이 앱을 먼저 실행하세요 ({why})"),
    )
}

struct Cli {
    cmd: String,
    pos: Vec<String>,
    json: bool,
    no_wait: bool,
    agent: Option<String>,
    conv: Option<i64>,
    /// 트랜잭션 키(`ask --id`).
    id: Option<String>,
}

impl Cli {
    fn parse(args: &[String]) -> Result<Self, Fail> {
        let bad = |msg: &str| usage(format!("{msg}\n{USAGE}"));
        let Some(cmd) = args.first().filter(|c| is_command(c)) else {
            return Err(bad("모르는 명령"));
        };
        let mut cli = Cli {
            cmd: cmd.clone(),
            pos: Vec::new(),
            json: false,
            no_wait: false,
            agent: None,
            conv: None,
            id: None,
        };
        let mut it = args[1..].iter();
        while let Some(a) = it.next() {
            if a == "--" {
                cli.pos.extend(it.by_ref().cloned());
                break;
            }
            match a.as_str() {
                "--json" => cli.json = true,
                "--no-wait" => cli.no_wait = true,
                "--agent" => {
                    cli.agent = Some(
                        it.next()
                            .ok_or_else(|| bad("--agent 에 이름이 없다"))?
                            .clone(),
                    )
                }
                "--id" => {
                    cli.id = Some(it.next().ok_or_else(|| bad("--id 에 키가 없다"))?.clone())
                }
                "--conv" => {
                    let v = it.next().ok_or_else(|| bad("--conv 에 대화 ID 가 없다"))?;
                    cli.conv = Some(v.parse().map_err(|_| bad("--conv 는 숫자 대화 ID"))?);
                }
                f if f.starts_with("--") => return Err(bad(&format!("모르는 옵션 {f}"))),
                _ => cli.pos.push(a.clone()),
            }
        }
        let (min, max, agent, no_wait, conv) = match cli.cmd.as_str() {
            "peers" => (0, 0, false, false, false),
            "ask" => (2, 2, true, true, true),
            "wait" | "status" => (1, 1, false, false, false),
            _ => (0, 1, false, false, true), // history
        };
        if cli.pos.len() < min || cli.pos.len() > max {
            return Err(bad(&format!("{} 의 인자 수가 맞지 않다", cli.cmd)));
        }
        if (cli.agent.is_some() && !agent)
            || (cli.no_wait && !no_wait)
            || (cli.conv.is_some() && !conv)
            || (cli.id.is_some() && cli.cmd != "ask")
        {
            return Err(bad(&format!("{} 에 쓸 수 없는 옵션", cli.cmd)));
        }
        if min == 2 && cli.pos[1].trim().is_empty() {
            return Err(bad("본문이 비었다"));
        }
        if matches!(cli.cmd.as_str(), "wait" | "status") && cli.pos[0].parse::<i64>().is_err() {
            return Err(bad("트랜잭션은 숫자"));
        }
        Ok(cli)
    }

    fn exec(
        &self,
        api: &Api,
        out: &mut dyn Write,
        err: &mut dyn Write,
        wait_limit: Duration,
    ) -> Result<(), Fail> {
        match self.cmd.as_str() {
            "peers" => self.peers(api, out),
            "ask" => self.ask(api, out, err, wait_limit),
            "wait" => self.wait(api, out, err, wait_limit),
            "status" => self.status(api, out),
            _ => self.history(api, out),
        }
    }

    fn peers(&self, api: &Api, out: &mut dyn Write) -> Result<(), Fail> {
        let peers = api.call("GET", "/api/peers", None)?;
        if self.json {
            return line(out, &peers);
        }
        for p in list(&peers) {
            let state = if p["paired"] == true {
                "페어링됨"
            } else {
                "대기"
            };
            line(
                out,
                format!(
                    "{}\t{}\t{}\t{state}",
                    text(&p["name"]),
                    text(&p["host"]),
                    hex(text(&p["fingerprint"]))
                        .chars()
                        .take(8)
                        .collect::<String>()
                ),
            )?;
        }
        Ok(())
    }

    fn ask(
        &self,
        api: &Api,
        out: &mut dyn Write,
        err: &mut dyn Write,
        wait_limit: Duration,
    ) -> Result<(), Fail> {
        let peer = resolve_peer(api, &self.pos[0])?;
        let (pid, question) = (peer["id"].as_i64().unwrap_or(0), self.pos[1].as_str());
        let sent = if let Some(cid) = self.conv {
            let c = conversation_of(api, cid, &peer)?;
            if c["dir"] != "out" {
                return Err(usage(format!(
                    "대화 {cid} 는 상대가 연 대화다 — 상대 에이전트에게 묻는 대화가 아니다"
                )));
            }
            // --agent 가 대화 에이전트가 아니면 멘션 — 자동 왕복 대화는 지금처럼 오류, 상대 PC 에 없는 이름은 사용법 오류(기록을 남기지 않게).
            let mention = self.agent.as_deref().filter(|a| *a != text(&c["agent"]));
            if let Some(a) = mention {
                if !c["relay_agent"].is_null() {
                    return Err(usage(format!(
                        "대화 {cid} 의 에이전트는 {} 다(--agent {a} 와 다르다)",
                        text(&c["agent"])
                    )));
                }
                let r = api.call("GET", &format!("/api/peers/{pid}/agents"), None)?;
                let known = r["names"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|n| n.as_str() == Some(a));
                if !known {
                    return Err(usage(format!(
                        "{} 에 에이전트 {a} 가 없다 — 멘션은 상대 PC 에이전트 이름",
                        text(&peer["name"])
                    )));
                }
            }
            let body = json!({ "text": question, "to": "agent", "via": "cli", "agent": mention, "tx_key": self.id });
            api.call(
                "POST",
                &format!("/api/conversations/{cid}/messages"),
                Some(body),
            )?
        } else {
            let agent = match &self.agent {
                Some(a) => a.clone(),
                None => sole_agent(api, pid, text(&peer["name"]))?,
            };
            let body = json!({ "peer_id": pid, "agent": agent, "text": question, "via": "cli", "tx_key": self.id });
            api.call("POST", "/api/conversations", Some(body))?
        };
        let (conv, asked) = (&sent["conversation"], &sent["message"]);
        let cid = conv["id"].as_i64().unwrap_or(0);
        let tx = sent["tx"].as_i64();
        if self.no_wait {
            return if self.json {
                line(out, json!({ "conversation": conv, "asked": asked, "tx": tx }))
            } else {
                line(out, cid)
            };
        }
        let tx = tx.ok_or_else(|| Fail(EXIT_API, "앱이 트랜잭션을 만들지 못했다 — 다시 보내지 말고 history 로 확인".into()))?;
        let done = wait_tx(api, tx, wait_limit, self.json, err)?;
        self.answer(out, &done)
    }

    /// 이어 받기 — 멈춘 트랜잭션도 다시 감시한다.
    fn wait(
        &self,
        api: &Api,
        out: &mut dyn Write,
        err: &mut dyn Write,
        wait_limit: Duration,
    ) -> Result<(), Fail> {
        let tx: i64 = self.pos[0].parse().unwrap_or(0);
        api.call("POST", &format!("/api/tx/{tx}/resume"), Some(json!({})))?;
        let done = wait_tx(api, tx, wait_limit, self.json, err)?;
        self.answer(out, &done)
    }

    /// 지금 상태 한 번.
    fn status(&self, api: &Api, out: &mut dyn Write) -> Result<(), Fail> {
        let v = api.call("GET", &format!("/api/tx/{}", self.pos[0]), None)?;
        if self.json {
            return line(out, &v);
        }
        let state = text(&v["state"]);
        let more = progress_line(&v).unwrap_or_default();
        line(out, format!("{}\t{state}\t{more}", self.pos[0]).trim_end())
    }

    /// 끝난 트랜잭션 출력 — 답 본문 또는 JSON `{conversation, asked, reply, tx}`.
    fn answer(&self, out: &mut dyn Write, done: &Value) -> Result<(), Fail> {
        if self.json {
            line(
                out,
                json!({ "conversation": done["conversation"], "asked": done["message"], "reply": done["reply"], "tx": done["tx"] }),
            )
        } else {
            line(out, text(&done["reply"]["text"]))
        }
    }

    fn history(&self, api: &Api, out: &mut dyn Write) -> Result<(), Fail> {
        let peers = api.call("GET", "/api/peers", None)?;
        let peer = match self.pos.first() {
            Some(q) => Some(resolve_in(&peers, q)?),
            None => None,
        };
        let name_of = |id: &Value| {
            list(&peers)
                .into_iter()
                .find(|p| p["id"] == *id)
                .map_or("상대".to_string(), |p| text(&p["name"]).to_string())
        };
        let convs = list(&api.call("GET", "/api/conversations", None)?);
        if let Some(cid) = self.conv {
            let c = convs
                .into_iter()
                .find(|c| c["id"] == cid)
                .ok_or_else(|| usage(format!("대화 {cid} 가 없다")))?;
            if let Some(p) = peer.as_ref().filter(|p| p["id"] != c["peer_id"]) {
                return Err(usage(format!(
                    "대화 {cid} 는 {} 와의 대화가 아니다",
                    text(&p["name"])
                )));
            }
            let ms = api.call("GET", &format!("/api/conversations/{cid}/messages"), None)?;
            if self.json {
                return line(out, &ms);
            }
            let who = name_of(&c["peer_id"]);
            for m in list(&ms) {
                let at = utc_time(m["created_at"].as_i64().unwrap_or(0));
                line(
                    out,
                    format!(
                        "[{at}] {}: {}",
                        author_label(&c, &m, &who),
                        text(&m["text"])
                    ),
                )?;
            }
            return Ok(());
        }
        let mine: Vec<Value> = convs
            .into_iter()
            .filter(|c| peer.as_ref().is_none_or(|p| p["id"] == c["peer_id"]))
            .collect();
        if self.json {
            return line(out, Value::Array(mine));
        }
        for c in &mine {
            line(
                out,
                format!(
                    "{}\t{}\t{}\t{}\t{}",
                    c["id"],
                    name_of(&c["peer_id"]),
                    text(&c["agent"]),
                    text(&c["title"]).replace(['\t', '\n', '\r'], " "),
                    text(&c["last_state"])
                ),
            )?;
        }
        Ok(())
    }
}

/// 주 인스턴스 로컬 API.
struct Api {
    info: InstanceInfo,
}

impl Api {
    fn open(instance_file: &Path) -> Result<Self, Fail> {
        match InstanceInfo::read(instance_file) {
            Ok(Some(info)) => Ok(Self { info }),
            Ok(None) => Err(not_running("instance.json 형식 오류")),
            Err(e) => Err(not_running(format!("instance.json: {e}"))),
        }
    }

    /// 보내기 전에 주 인스턴스가 이 CLI 를 아는지 본다 — 이전 버전은 `via` 를 모르고 사람 글로 남긴다(출처 표시가 조용히 사라진다).
    fn handshake(&self) -> Result<(), Fail> {
        let status = self.call("GET", "/api/status", None)?;
        if status["cli_api"].as_u64() == Some(u64::from(CLI_API)) {
            return Ok(());
        }
        Err(Fail(
            EXIT_API,
            format!(
                "실행 중인 coworkAgent 가 이 CLI 를 지원하지 않는다(이전 버전 또는 다른 프로그램, cli_api {} ≠ {CLI_API}) — 앱을 종료하고 이 실행 파일로 다시 실행하세요",
                status["cli_api"]
            ),
        ))
    }

    fn call(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value, Fail> {
        let body = body.map(|b| b.to_string());
        match local_request(&self.info, method, path, body.as_deref(), REQUEST_DEADLINE) {
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::AddrNotAvailable
                ) =>
            {
                Err(not_running(e))
            }
            // 다른 프로세스가 남은 포트를 쓰고 있으면 HTTP 응답이 아니다 — 우리 앱이 아니다.
            Err(e) if e.kind() == io::ErrorKind::InvalidData => Err(not_running(e)),
            // 보낸 뒤 응답을 못 받았다 — 앱이 저장 · 전송했을 수 있다. "실행 중 아님"(3)으로 알리면 다시 보내 두 번 실행된다.
            Err(e)
                if method == "POST"
                    && matches!(e.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::TimedOut) =>
            {
                Err(Fail(
                    EXIT_API,
                    format!("{e}\n전달 여부를 모른다 — 다시 보내지 말고 coworkAgent history <상대> 로 확인하세요"),
                ))
            }
            Err(e) => Err(Fail(EXIT_API, format!("로컬 API 요청 실패: {e}"))),
            Ok((code, body)) if (200..300).contains(&code) => serde_json::from_str(&body)
                .map_err(|e| Fail(EXIT_API, format!("로컬 API 응답이 JSON 이 아니다({code}): {e}"))),
            Ok((401, _)) => Err(Fail(
                EXIT_API,
                "로컬 API 가 토큰을 거부했다(401) — instance.json 이 지금 실행 중인 coworkAgent 의 것이 아니다".into(),
            )),
            Ok((code, body)) => {
                let why = serde_json::from_str::<Value>(&body)
                    .ok()
                    .and_then(|v| v["error"].as_str().map(str::to_string))
                    .unwrap_or(body);
                // 502 = 상대와의 전송 실패 — 상대가 이미 받았을 수 있다(다시 보내면 두 번 실행될 수 있다).
                let hint = if code == 502 && method == "POST" {
                    "\n상대에게 전달됐을 수 있다 — 다시 보내기 전에 coworkAgent history <상대> 로 확인하세요"
                } else {
                    ""
                };
                Err(Fail(EXIT_API, format!("로컬 API 거부({code}): {why}{hint}")))
            }
        }
    }
}

/// 페어링된 상대 하나 — 이름 완전 일치(대소문자 무시) 또는 지문 hex 앞자리(4자 이상).
fn resolve_peer(api: &Api, query: &str) -> Result<Value, Fail> {
    resolve_in(&api.call("GET", "/api/peers", None)?, query)
}

fn resolve_in(peers: &Value, query: &str) -> Result<Value, Fail> {
    let q = query.trim();
    let qhex = hex(q);
    let by_fp = qhex.len() >= 4
        && qhex.len() == q.chars().filter(|c| !matches!(c, ' ' | '-' | ':')).count();
    let hits: Vec<Value> = list(peers)
        .into_iter()
        .filter(|p| p["paired"] == true)
        .filter(|p| {
            text(&p["name"]).to_lowercase() == q.to_lowercase()
                || (by_fp && hex(text(&p["fingerprint"])).starts_with(&qhex))
        })
        .collect();
    match hits.len() {
        0 => Err(usage(format!(
            "페어링된 상대 '{q}' 가 없다 — coworkAgent peers 로 확인"
        ))),
        1 => Ok(hits.into_iter().next().unwrap_or_default()),
        _ => Err(usage(format!(
            "'{q}' 에 맞는 상대가 여럿 — 지문 앞자리로 고르세요: {}",
            hits.iter()
                .map(|p| format!(
                    "{} ({})",
                    text(&p["name"]),
                    hex(text(&p["fingerprint"]))
                        .chars()
                        .take(8)
                        .collect::<String>()
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

/// 그 상대와의 대화 `cid`(요약 JSON).
fn conversation_of(api: &Api, cid: i64, peer: &Value) -> Result<Value, Fail> {
    let c = list(&api.call("GET", "/api/conversations", None)?)
        .into_iter()
        .find(|c| c["id"] == cid)
        .ok_or_else(|| usage(format!("대화 {cid} 가 없다")))?;
    if c["peer_id"] != peer["id"] {
        return Err(usage(format!(
            "대화 {cid} 는 {} 와의 대화가 아니다",
            text(&peer["name"])
        )));
    }
    Ok(c)
}

/// `--agent` 가 없을 때 — 상대 에이전트가 정확히 1개면 그것.
fn sole_agent(api: &Api, pid: i64, name: &str) -> Result<String, Fail> {
    let r = api.call("GET", &format!("/api/peers/{pid}/agents"), None)?;
    let names: Vec<&str> = r["names"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    match names.as_slice() {
        [one] => Ok(one.to_string()),
        [] => Err(usage(format!("{name} 에 등록된 에이전트가 없다"))),
        many => Err(usage(format!(
            "{name} 의 에이전트가 여럿 — --agent 로 고르세요: {}",
            many.join(", ")
        ))),
    }
}

/// 트랜잭션이 끝날 때까지(#1c) — 진행(보내는 중 · 대기 · 처리 중 + 단계)이 바뀔 때마다 `err` 에 한 줄. 끝: 답(돌려줌) · 실패(5) · 멈춤(7) ·
/// 안전판(`limit` — 6). CLI 가 죽어도 트랜잭션은 앱에 남는다(`wait <tx>`).
fn wait_tx(api: &Api, tx: i64, limit: Duration, json: bool, err: &mut dyn Write) -> Result<Value, Fail> {
    let until = Instant::now() + limit;
    let mut shown: Option<String> = None;
    loop {
        let v = api
            .call("GET", &format!("/api/tx/{tx}"), None)
            .map_err(|Fail(code, msg)| Fail(code, format!("{msg}\n나중에 이어 받기: coworkAgent wait {tx}")))?;
        let cid = v["conversation"]["id"].as_i64().unwrap_or(0);
        match text(&v["state"]) {
            "answered" => return Ok(v),
            "failed" => {
                let why = if v["reply"].is_object() {
                    format!("상대 에이전트 실행 실패: {}", text(&v["reply"]["text"]))
                } else {
                    format!(
                        "질문을 상대에 보내지 못했다({})",
                        v["message"]["refused"].as_str().unwrap_or(text(&v["message"]["state"]))
                    )
                };
                return Err(Fail(EXIT_REPLY_FAILED, why));
            }
            "stalled" => {
                return Err(Fail(
                    EXIT_STALLED,
                    format!("상대에 닿지 않아 멈췄다 — 트랜잭션 {tx} 는 남는다. 나중에 이어 받기: coworkAgent wait {tx}"),
                ));
            }
            _ => {}
        }
        let now = if json { progress_json(&v).map(|j| j.to_string()) } else { progress_line(&v) };
        if let Some(l) = now.filter(|l| shown.as_ref() != Some(l)) {
            let _ = line(err, &l);
            shown = Some(l);
        }
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(Fail(
                EXIT_TIMEOUT,
                format!(
                    "답 대기 시간 초과 — 대화 {cid} 는 계속된다. 나중에: coworkAgent wait {tx} · coworkAgent history --conv {cid}"
                ),
            ));
        }
        std::thread::sleep(left.min(POLL));
    }
}

/// 진행 한 줄(평문) — 보내는 중 · 대기(앞 사람 수) · 처리 중(+ 단계 요약). 진행이 없으면 `None`.
fn progress_line(v: &Value) -> Option<String> {
    let p = &v["progress"];
    match text(&v["state"]) {
        "sending" => Some(format!(
            "보내는 중 — 상대에 아직 닿지 않아 이 PC 에 보관했다(다시 보냄 {}회)",
            v["outbox"]["tries"].as_i64().unwrap_or(0)
        )),
        "queued" => Some(format!(
            "대기 중 — 상대 PC 에 처리 중인 작업이 많다(앞에 {}건)",
            p["ahead"].as_u64().unwrap_or(0)
        )),
        "running" => {
            let d = &p["detail"];
            let mut parts = Vec::new();
            if let Some(t) = d["turns"].as_u64().filter(|t| *t > 0) {
                parts.push(format!("{t}턴"));
            }
            if let Some(tools) = d["tools"].as_object() {
                parts.extend(tools.iter().map(|(k, n)| format!("{k} {n}")));
            }
            if let Some(s) = d["secs"].as_u64().filter(|s| *s > 0) {
                parts.push(format!("{s}초"));
            }
            Some(if parts.is_empty() {
                "처리 중".into()
            } else {
                format!("처리 중 — {}", parts.join(" · "))
            })
        }
        _ => None,
    }
}

/// 진행 한 줄(`--json`) — `{"progress": "sending"|"queued"|"running", "ahead"?, "detail"?}`.
fn progress_json(v: &Value) -> Option<Value> {
    let p = &v["progress"];
    match text(&v["state"]) {
        "sending" => Some(json!({ "progress": "sending", "tries": v["outbox"]["tries"] })),
        "queued" => Some(json!({ "progress": "queued", "ahead": p["ahead"].as_u64().unwrap_or(0) })),
        "running" if p["detail"].is_object() => Some(json!({ "progress": "running", "detail": p["detail"] })),
        "running" => Some(json!({ "progress": "running" })),
        _ => None,
    }
}

/// 발화자 — 웹UI `view.js` `authorLabel` 과 같은 규칙(보는 쪽 기준).
fn author_label(c: &Value, m: &Value, peer: &str) -> String {
    let mine = m["mine"] == true;
    // 답 발화자 = 실행한 에이전트(멘션이면 부른 이름).
    let agent = m["agent"].as_str().unwrap_or(text(&c["agent"]));
    let relay = c["relay_agent"].as_str().unwrap_or("에이전트");
    if m["author"] == "agent" {
        return match (c["dir"] == "out", mine) {
            (true, true) => format!("내 {relay}"),
            (true, false) => format!("{peer}/{agent}"),
            (false, false) => format!("{peer}/{relay}"),
            (false, true) => format!("내 {agent}"),
        };
    }
    match (m["via"] == "cli", mine) {
        (true, true) => format!("내 {CLI_LABEL}"),
        (true, false) => format!("{peer}/{CLI_LABEL}"),
        (false, true) => "나".into(),
        (false, false) => peer.into(),
    }
}

/// 유닉스 초 → `YYYY-MM-DD HH:MM UTC`(그레고리력, 시간대 의존 없음).
fn utc_time(secs: i64) -> String {
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02} UTC",
        rem / 3600,
        rem % 3600 / 60
    )
}

fn list(v: &Value) -> Vec<Value> {
    v.as_array().cloned().unwrap_or_default()
}

fn text(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}

/// 지문 표기 → hex 대문자(공백 · `-` · `:` 제거).
fn hex(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, ' ' | '-' | ':'))
        .map(|c| c.to_ascii_uppercase())
        .filter(char::is_ascii_hexdigit)
        .collect()
}

fn line(out: &mut dyn Write, s: impl std::fmt::Display) -> Result<(), Fail> {
    writeln!(out, "{}", printable(&s.to_string()))
        .map_err(|e| Fail(EXIT_API, format!("출력 실패: {e}")))
}

/// 터미널을 조작하는 제어 문자를 지운다 — C0(줄바꿈 · 탭 제외) · DEL · C1. 원격 답 · 이름이 화면 · 파이프로 나간다(1a A14).
pub fn printable(s: &str) -> String {
    s.chars()
        .filter(|&c| c == '\n' || c == '\t' || !c.is_control())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_time_known_dates() {
        assert_eq!(utc_time(0), "1970-01-01 00:00 UTC");
        assert_eq!(utc_time(951_782_400), "2000-02-29 00:00 UTC");
        assert_eq!(utc_time(1_790_000_000), "2026-09-21 14:13 UTC");
    }

    #[test]
    fn fingerprint_query_needs_hex_only() {
        let peers = json!([
            { "id": 1, "name": "dead", "fingerprint": "AB12 CD34", "paired": true },
            { "id": 2, "name": "bob", "fingerprint": "DEAD BEEF", "paired": true },
            { "id": 3, "name": "pending", "fingerprint": "AB12 0000", "paired": false },
        ]);
        assert_eq!(resolve_in(&peers, "BOB").unwrap()["id"], 2);
        assert_eq!(
            resolve_in(&peers, "ab12").unwrap()["id"],
            1,
            "대기 중인 상대는 고르지 않는다"
        );
        assert_eq!(
            resolve_in(&peers, "dead").err().map(|f| f.0),
            Some(EXIT_USAGE),
            "이름 · 지문 둘 다 맞으면 모호"
        );
        assert_eq!(
            resolve_in(&peers, "ab1").err().map(|f| f.0),
            Some(EXIT_USAGE),
            "4자 미만 지문은 쓰지 않는다"
        );
        assert_eq!(
            resolve_in(&peers, "xyz-dead").err().map(|f| f.0),
            Some(EXIT_USAGE),
            "hex 아닌 글자가 섞이면 지문이 아니다"
        );
    }
}
