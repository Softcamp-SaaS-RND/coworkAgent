//! 화면 토큰 강제(plan 해석 10 · 20) — `app.css` 첫 `:root` 밖에서 색 · 글꼴 · 글꼴 크기 · 굵기 · 줄높이 · 둥글기 · 간격은 `var(--…)` 만 쓴다.
//! 예외: `0` · `auto` · `1px`(테두리 — `border*` 는 색 종류라 길이를 보지 않는다) · `-1px`(시각적 숨김 — `.sr-only` 한 곳) · `50%`(원) ·
//! `inherit` · `line-height: 1`(아이콘 상자) · `@font-face` 블록 · `url(…)` 안. `var(--이름, 대체)` 의 대체 값도 검사하고, 첫 `:root` 밖
//! 사용자 속성(`--이름: …`)은 `var(…)` · `url(…)` 만 — 지역 변수로 리터럴을 숨기지 못한다.
const CSS: &str = include_str!("../web/app.css");

/// 검사하는 속성 종류.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    Color,
    Family,
    Size,
    Weight,
    LineHeight,
    Radius,
    Space,
    /// 첫 `:root` 밖 사용자 속성(`--st` · `--tone` · `--ico`) — `var(…)` · `url(…)` 만(값 종류를 모르니 무엇이든 리터럴이면 막는다).
    Custom,
}

fn kind(prop: &str) -> Option<Kind> {
    let p = prop.trim();
    Some(match p {
        "color"
        | "background"
        | "background-color"
        | "outline"
        | "outline-color"
        | "box-shadow"
        | "fill"
        | "stroke"
        | "accent-color"
        | "caret-color"
        | "text-decoration-color" => Kind::Color,
        "font-family" | "font" => Kind::Family,
        "font-size" => Kind::Size,
        "font-weight" => Kind::Weight,
        "line-height" => Kind::LineHeight,
        "gap" | "row-gap" | "column-gap" => Kind::Space,
        _ if p.starts_with("--") => Kind::Custom,
        _ if p.ends_with("radius") => Kind::Radius,
        _ if p.starts_with("border") => Kind::Color,
        _ if p.starts_with("padding") || p.starts_with("margin") => Kind::Space,
        _ => return None,
    })
}

/// 주석을 지운다.
fn strip_comments(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find("/*") {
        out.push_str(&rest[..i]);
        rest = rest[i..].find("*/").map_or("", |j| &rest[i + j + 2..]);
    }
    out + rest
}

/// `url(…)` 안(데이터 URI 아이콘 등)을 지운다 — 따옴표 안 괄호까지.
fn strip_urls(v: &str) -> String {
    let mut out = String::new();
    let mut rest = v;
    while let Some(i) = rest.find("url(") {
        out.push_str(&rest[..i]);
        let body = &rest[i + 4..];
        let end = match body.chars().next() {
            Some(q @ ('"' | '\'')) => body[1..].find(q).map(|j| j + 2),
            _ => Some(0),
        }
        .and_then(|from| body[from..].find(')').map(|k| from + k + 1));
        rest = end.map_or("", |e| &body[e..]);
    }
    out + rest
}

/// 첫 `:root { … }` 를 뗀 나머지에서 가장 안쪽 블록의 (선택자, 속성, 값). `@font-face` 블록은 뺀다.
fn declarations() -> Vec<(String, String, String)> {
    let css = strip_comments(CSS);
    let start = css.find(":root").expect(":root");
    let end = start + css[start..].find('}').expect(":root 끝") + 1;
    let rest = format!("{}{}", &css[..start], &css[end..]);
    let mut out = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut last = 0;
    for (i, c) in rest.char_indices() {
        match c {
            '{' => {
                stack.push(i);
                last = i + 1;
            }
            '}' => {
                let open = stack.pop().expect("짝 없는 }");
                let head_start = rest[..open].rfind(['}', '{', ';']).map_or(0, |k| k + 1);
                let selector = rest[head_start..open].trim().to_string();
                if last == open + 1 && !selector.starts_with("@font-face") {
                    for decl in rest[open + 1..i].split(';') {
                        if let Some((p, v)) = decl.split_once(':') {
                            out.push((
                                selector.clone(),
                                p.trim().to_string(),
                                v.trim().to_string(),
                            ));
                        }
                    }
                }
                last = i + 1;
            }
            _ => {}
        }
    }
    out
}

/// 값 조각 — `var(--이름)` 은 하나로(대체 값은 다시 조각으로 — 대체에 리터럴을 숨기지 못하게), 괄호 · 쉼표 · 연산자는 가른다.
fn tokens(v: &str) -> Vec<String> {
    let v = strip_urls(v);
    let mut out = Vec::new();
    let mut rest = v.as_str();
    while let Some(i) = rest.find("var(") {
        out.extend(plain(&rest[..i]));
        let body = &rest[i + 4..];
        let mut depth = 0;
        let close = body
            .char_indices()
            .find(|&(_, c)| {
                match c {
                    '(' => depth += 1,
                    ')' if depth == 0 => return true,
                    ')' => depth -= 1,
                    _ => {}
                }
                false
            })
            .map_or(body.len(), |(k, _)| k);
        let (name, fallback) = body[..close]
            .split_once(',')
            .unwrap_or((&body[..close], ""));
        out.push(format!("var({})", name.trim()));
        out.extend(tokens(fallback));
        rest = body.get(close + 1..).unwrap_or("");
    }
    out.extend(plain(rest));
    out
}

fn plain(s: &str) -> Vec<String> {
    s.split(|c: char| c.is_whitespace() || "(),*/+".contains(c))
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

fn has_unit(t: &str) -> bool {
    let t = t.trim_start_matches('-');
    t.starts_with(|c: char| c.is_ascii_digit() || c == '.')
        && t.ends_with(|c: char| c.is_ascii_alphabetic() || c == '%')
}

fn is_number(t: &str) -> bool {
    !t.is_empty()
        && t.trim_start_matches('-')
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
}

const NAMED: [&str; 12] = [
    "white", "black", "red", "green", "blue", "gray", "grey", "orange", "yellow", "purple", "pink",
    "silver",
];

/// 리터럴이면 그 조각.
fn literal(kind: Kind, value: &str) -> Option<String> {
    let toks = tokens(value);
    if kind == Kind::Family {
        return (!matches!(value, "var(--font)" | "var(--mono)" | "inherit"))
            .then(|| value.to_string());
    }
    toks.into_iter().find(|t| {
        if t.starts_with("var(") || t == "inherit" || t == "0" {
            return false;
        }
        match kind {
            Kind::Color => {
                t.starts_with('#')
                    || ["rgb", "rgba", "hsl", "hsla"].contains(&t.as_str())
                    || NAMED.contains(&t.to_ascii_lowercase().as_str())
            }
            Kind::Size => has_unit(t) || is_number(t),
            Kind::Weight => {
                is_number(t) || ["bold", "bolder", "lighter", "normal"].contains(&t.as_str())
            }
            Kind::LineHeight => (has_unit(t) || is_number(t)) && t != "1",
            Kind::Radius => has_unit(t) && t != "50%" && t != "0px",
            Kind::Space => has_unit(t) && t != "0px",
            Kind::Custom => true,
            Kind::Family => unreachable!(),
        }
    })
}

/// 시각적 숨김(`.sr-only`)의 `margin: -1px` — 간격 리터럴의 하나뿐인 예외(plan 해석 10).
const HIDDEN: (&str, &str, &str) = (".sr-only", "margin", "-1px");

#[test]
fn literals_only_in_root() {
    let decls = declarations();
    let (hidden, checked): (Vec<_>, Vec<_>) = decls
        .iter()
        .filter_map(|(s, p, v)| kind(p).map(|k| (s, p, v, k)))
        .partition(|(s, p, v, _)| (s.as_str(), p.as_str(), v.as_str()) == HIDDEN);
    assert_eq!(
        hidden.len(),
        1,
        "시각적 숨김 예외는 `.sr-only {{ margin: -1px }}` 한 곳"
    );
    assert!(
        checked.len() >= 300,
        "검사한 선언 {} — 300 이상이어야 한다(파서가 선언을 놓친다)",
        checked.len()
    );
    let bad: Vec<String> = checked
        .iter()
        .filter_map(|(s, p, v, k)| literal(*k, v).map(|t| format!("{s} {{ {p}: {v} }} — {t}")))
        .collect();
    assert!(
        bad.is_empty(),
        "첫 :root 밖 리터럴 {} 곳:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

/// `var(--이름)` 은 CSS 어딘가에서 선언된 이름이다 — 철자가 틀리면 값이 조용히 빠진다(`--side-w` 는 app.js 가 넣는다).
#[test]
fn every_var_is_declared() {
    let css = strip_comments(CSS);
    let declared: Vec<&str> = css
        .match_indices("--")
        .filter_map(|(i, _)| {
            let name = &css[i..];
            let end = name.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))?;
            name[end..]
                .trim_start()
                .starts_with(':')
                .then(|| &name[..end])
        })
        .collect();
    let missing: Vec<&str> = css
        .match_indices("var(--")
        .map(|(i, _)| {
            let name = &css[i + 4..];
            &name[..name.find([')', ',']).unwrap_or(name.len())]
        })
        .filter(|n| *n != "--side-w" && !declared.contains(&n.trim()))
        .collect();
    assert!(missing.is_empty(), "선언 없는 토큰: {missing:?}");
}

#[test]
fn literal_rules_catch_literals() {
    let cases = [
        (Kind::Space, "0 12px", Some("12px")),
        (Kind::Space, "0 var(--sp-3) auto", None),
        (Kind::Space, "1px var(--sp-3)", Some("1px")),
        (Kind::Space, "-1px", Some("-1px")),
        (Kind::Space, "calc(var(--sp-7) + var(--sp-5))", None),
        (Kind::Color, "1px solid #fff", Some("#fff")),
        (Kind::Color, "1px solid var(--border)", None),
        (Kind::Color, "var(--danger, #e5484d)", Some("#e5484d")),
        (Kind::Color, "var(--tone, var(--wait))", None),
        (Kind::Color, "0 0 0 2px var(--st)", None),
        (
            Kind::Color,
            "url(\"data:image/svg+xml,%3Csvg stroke='black'%3E\")",
            None,
        ),
        (Kind::Color, "rgba(0,0,0,.4)", Some("rgba")),
        (Kind::Weight, "600", Some("600")),
        (Kind::LineHeight, "1", None),
        (Kind::LineHeight, "1.5", Some("1.5")),
        (Kind::Radius, "50%", None),
        (Kind::Radius, "4px var(--radius-bubble)", Some("4px")),
        (Kind::Size, "11px", Some("11px")),
        (
            Kind::Family,
            "'Noto Sans KR', sans-serif",
            Some("'Noto Sans KR', sans-serif"),
        ),
    ];
    for (k, v, want) in cases {
        assert_eq!(literal(k, v).as_deref(), want, "{k:?} {v}");
    }
    // 지역 사용자 속성으로 리터럴을 숨기지 못한다 — `.x { --pad: 1px; padding: var(--pad) }`(#1p 적대적 리뷰 라운드 2).
    let local = [
        ("--pad", "1px", Some("1px")),
        ("--r", "6px", Some("6px")),
        ("--fs", "13px", Some("13px")),
        ("--c", "#fff", Some("#fff")),
        ("--w", "600", Some("600")),
        ("--st", "var(--ok)", None),
        ("--ico", "url(\"data:image/svg+xml,%3Csvg%3E\")", None),
    ];
    for (prop, v, want) in local {
        let k = kind(prop).expect(prop);
        assert_eq!(literal(k, v).as_deref(), want, "{prop}: {v}");
    }
}
