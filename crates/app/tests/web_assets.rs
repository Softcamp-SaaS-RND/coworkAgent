//! 화면 모듈 표(`server::SCRIPTS`) — `web/js/` 의 파일과 같고, 상대 `import` 대상이 표에 있고, 가져오는 이름을 대상이 `export` 한다.
//! 진입은 `core/token.js` 를 맨 먼저 평가하고(토큰이 없으면 다른 모듈의 최상위 코드가 돌지 않는다), DOM 없는 `logic/` 은 UI 모듈을 부르지 않는다.
use an_app::server::SCRIPTS;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// 모듈의 `import` 문 — (대상 모듈 이름, 가져오는 이름들). 대상은 이 모듈 기준 상대 경로를 푼 것.
fn imports(module: &str, body: &str) -> Vec<(String, Vec<String>)> {
    let body = body.replace("\r\n", "\n");
    let mut out = Vec::new();
    let mut rest = body.as_str();
    loop {
        let at = match rest.find("\nimport ") {
            _ if rest.starts_with("import ") => 0,
            Some(i) => i + 1,
            None => break,
        };
        let stmt = &rest[at..at + rest[at..].find(';').expect("import 끝 ;")];
        rest = &rest[at + stmt.len()..];
        let target = stmt.rsplit('"').nth(1).expect("import 대상");
        let names = match (stmt.find('{'), stmt.find('}')) {
            (Some(a), Some(b)) => stmt[a + 1..b].split(',').map(|n| n.trim().to_string()).filter(|n| !n.is_empty()).collect(),
            _ => Vec::new(),
        };
        out.push((resolve(module, target), names));
    }
    out
}

fn resolve(module: &str, target: &str) -> String {
    assert!(target.starts_with("./") || target.starts_with("../"), "{module}: 상대 경로가 아닌 import {target}");
    let mut parts: Vec<&str> = module.split('/').collect();
    parts.pop();
    for seg in target.split('/') {
        match seg {
            "." => {}
            ".." => {
                assert!(parts.pop().is_some(), "{module}: js/ 밖을 가리키는 import {target}");
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// 최상위 `export` 선언 이름.
fn exports(body: &str) -> BTreeSet<String> {
    body.lines()
        .filter_map(|l| l.strip_prefix("export "))
        .map(|l| l.trim_start_matches("async "))
        .filter_map(|l| ["function* ", "function ", "const ", "let ", "var ", "class "].iter().find_map(|k| l.strip_prefix(k)))
        .map(|l| l.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$').collect())
        .collect()
}

fn files(dir: &Path, base: &Path, out: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, base, out);
        } else {
            out.insert(path.strip_prefix(base).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn scripts_match_files() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("web/js");
    let mut on_disk = BTreeSet::new();
    files(&base, &base, &mut on_disk);
    let listed: BTreeSet<String> = SCRIPTS.iter().map(|(name, _)| name.to_string()).collect();
    assert_eq!(listed.len(), SCRIPTS.len(), "표에 같은 이름이 두 번");
    assert_eq!(on_disk, listed, "web/js 파일 = SCRIPTS 이름");
    assert!(include_str!("../web/index.html").contains(r#"<script src="/js/app.js" type="module"></script>"#));
}

#[test]
fn imports_resolve_to_exports() {
    let table: BTreeMap<&str, BTreeSet<String>> = SCRIPTS.iter().map(|(name, body)| (*name, exports(body))).collect();
    let mut bad = Vec::new();
    for (module, body) in SCRIPTS {
        for (target, names) in imports(module, body) {
            match table.get(target.as_str()) {
                None => bad.push(format!("{module}: 표에 없는 대상 {target}")),
                Some(have) => bad.extend(names.iter().filter(|n| !have.contains(*n)).map(|n| format!("{module}: {target} 가 내보내지 않는 {n}"))),
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn token_first_and_layers() {
    let body = |name: &str| SCRIPTS.iter().find(|(n, _)| *n == name).unwrap().1;
    let entry = imports("app.js", body("app.js"));
    assert_eq!(entry[0], ("core/token.js".to_string(), Vec::new()), "진입의 첫 import = core/token.js");
    for (module, src) in SCRIPTS {
        let only: &[&str] = if module.starts_with("logic/") {
            &["logic/"]
        } else if ["core/dom.js", "core/state.js", "core/token.js"].contains(module) {
            &["core/", "logic/"]
        } else {
            continue;
        };
        for (target, _) in imports(module, src) {
            assert!(only.iter().any(|p| target.starts_with(p)), "{module} → {target}: {only:?} 만 import");
        }
    }
}
