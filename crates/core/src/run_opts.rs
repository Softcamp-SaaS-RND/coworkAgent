//! `claude -p` 실행 옵션(model · effort) · 그룹 이름 · 표시 이름 · 대화 제목 · 타일 아이콘 · 화면 설정(테마 · 사이드바 폭) —
//! 값 검증과 기본값의 단일 지점.
use crate::{peers::Peer, settings};
use rusqlite::Connection;

pub const DEFAULT_MODEL: &str = "sonnet";
pub const DEFAULT_EFFORT: &str = "high";
/// 고를 수 있는 모델 — Claude Code 별칭(#1d, 웹UI 드롭다운과 같다).
pub const MODELS: [&str; 3] = ["opus", "sonnet", "haiku"];
/// `claude --effort <level>` 이 받는 값(2.1.296 `--help`).
pub const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];
pub const MODEL_KEY: &str = "claude_model";
pub const EFFORT_KEY: &str = "claude_effort";
/// 화면 테마 — `dark`(기본) · `light`.
pub const THEME_KEY: &str = "ui_theme";
pub const THEMES: [&str; 2] = ["light", "dark"];
/// 레일 타일 순서(#1l) — `peers_first`(기본, 상대 → 그룹) · `groups_first`.
pub const RAIL_ORDER_KEY: &str = "ui_rail_order";
pub const RAIL_ORDERS: [&str; 2] = ["peers_first", "groups_first"];
/// 그룹 이름 · 표시 이름(별칭) 최대 글자 수.
pub const MAX_GROUP_CHARS: usize = 32;
pub const MAX_ALIAS_CHARS: usize = 32;
/// 사용자가 붙인 대화 제목 최대 글자 수(붙이지 않으면 첫 사람 글 앞 [`crate::chat::TITLE_CHARS`] 자).
pub const MAX_TITLE_CHARS: usize = 100;
/// 타일 아이콘 — 웹UI 가 고른 그림을 줄여 만든 PNG data URL 만(PNG 서명의 base64 로 시작), 이 글자 수까지.
pub const ICON_PREFIX: &str = "data:image/png;base64,iVBORw0KGgo";
pub const MAX_ICON_CHARS: usize = 65_536;
/// 사이드바(대화 목록) 폭(px).
pub const SIDE_WIDTH_KEY: &str = "ui_side_width";
pub const SIDE_WIDTHS: std::ops::RangeInclusive<i64> = 200..=560;
pub const DEFAULT_SIDE_WIDTH: i64 = 272;
/// 접은 절(#1g) — 웹UI 상대별 절 열쇠 `{화면}:{상대 id}`('내 에이전트' = `{화면}:me`)를 쉼표로 이은 한 값, 이 개수 · 글자 수까지.
pub const FOLDED_KEY: &str = "ui_folded";
pub const MAX_FOLDED: usize = 300;
pub const MAX_FOLD_CHARS: usize = 32;
/// '나' 타일 — 표시 이름 · 아이콘(빈 값 = 없음).
pub const ME_ALIAS_KEY: &str = "me_alias";
pub const ME_ICON_KEY: &str = "me_icon";

/// model — [`MODELS`] 중 하나.
pub fn valid_model(s: &str) -> bool {
    MODELS.contains(&s)
}

pub fn valid_effort(s: &str) -> bool {
    EFFORTS.contains(&s)
}

/// 사용자가 붙이는 짧은 이름 — 앞뒤 공백을 지우고 비면 `None`, `max` 자를 넘거나 제어 문자가 있으면 `Err`.
pub fn clean_label(s: &str, max: usize, what: &str) -> Result<Option<String>, String> {
    let s = s.trim();
    if s.chars().count() > max || s.chars().any(char::is_control) {
        return Err(format!("{what}은 {max}자까지, 제어 문자 없이"));
    }
    Ok((!s.is_empty()).then(|| s.to_string()))
}

pub fn clean_group(s: &str) -> Result<Option<String>, String> {
    clean_label(s, MAX_GROUP_CHARS, "그룹 이름")
}

/// 이 PC 에서 부르는 상대 · 내 표시 이름(타일 · 사이드바 머리 · 대화 목록).
pub fn clean_alias(s: &str) -> Result<Option<String>, String> {
    clean_label(s, MAX_ALIAS_CHARS, "표시 이름")
}

pub fn clean_title(s: &str) -> Result<Option<String>, String> {
    clean_label(s, MAX_TITLE_CHARS, "대화 제목")
}

/// 타일 아이콘 — 빈 문자열이면 `None`(지움), PNG data URL(서명 · base64 글자 · 길이)이 아니면 `Err`.
pub fn clean_icon(s: &str) -> Result<Option<String>, String> {
    if s.is_empty() {
        return Ok(None);
    }
    let ok = s.len() <= MAX_ICON_CHARS
        && s.strip_prefix(ICON_PREFIX).is_some_and(|rest| {
            rest.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))
        });
    if !ok {
        return Err(format!("아이콘은 PNG data URL({MAX_ICON_CHARS}자까지)"));
    }
    Ok(Some(s.to_string()))
}

// 아래 설정 읽기는 없거나 틀린 값만 기본값으로 — DB 오류는 그대로 낸다(기본값을 성공처럼 내지 않는다).

/// 전역 기본값 — 설정에 없거나 틀린 값이면 [`DEFAULT_MODEL`] · [`DEFAULT_EFFORT`].
pub fn defaults(conn: &Connection) -> rusqlite::Result<(String, String)> {
    let model = settings::get(conn, MODEL_KEY)?.filter(|m| valid_model(m));
    let effort = settings::get(conn, EFFORT_KEY)?.filter(|e| valid_effort(e));
    Ok((
        model.unwrap_or_else(|| DEFAULT_MODEL.into()),
        effort.unwrap_or_else(|| DEFAULT_EFFORT.into()),
    ))
}

/// 실행 옵션 — 상대의 덮어쓰기(맞는 값만) → 전역 기본값.
pub fn resolve(conn: &Connection, peer: Option<&Peer>) -> rusqlite::Result<(String, String)> {
    let (model, effort) = defaults(conn)?;
    let pick = |own: Option<&String>, ok: fn(&str) -> bool, base: String| {
        own.filter(|v| ok(v)).cloned().unwrap_or(base)
    };
    Ok((
        pick(peer.and_then(|p| p.model.as_ref()), valid_model, model),
        pick(peer.and_then(|p| p.effort.as_ref()), valid_effort, effort),
    ))
}

/// 화면 테마 — 없거나 틀리면 `dark`.
pub fn theme(conn: &Connection) -> rusqlite::Result<String> {
    Ok(settings::get(conn, THEME_KEY)?
        .filter(|t| THEMES.contains(&t.as_str()))
        .unwrap_or_else(|| "dark".into()))
}

/// 레일 타일 순서 — 없거나 틀리면 `peers_first`.
pub fn rail_order(conn: &Connection) -> rusqlite::Result<String> {
    Ok(settings::get(conn, RAIL_ORDER_KEY)?
        .filter(|t| RAIL_ORDERS.contains(&t.as_str()))
        .unwrap_or_else(|| RAIL_ORDERS[0].into()))
}

/// 사이드바 폭 — 없거나 범위 밖이면 [`DEFAULT_SIDE_WIDTH`].
pub fn side_width(conn: &Connection) -> rusqlite::Result<i64> {
    Ok(settings::get(conn, SIDE_WIDTH_KEY)?
        .and_then(|v| v.parse().ok())
        .filter(|w| SIDE_WIDTHS.contains(w))
        .unwrap_or(DEFAULT_SIDE_WIDTH))
}

/// 접은 절 열쇠 목록 → 저장 값. 열쇠는 `^[a-z]+:([0-9]+|me)$` · [`MAX_FOLD_CHARS`] 자까지, [`MAX_FOLDED`] 개까지.
pub fn clean_folded(keys: &[String]) -> Result<String, String> {
    let ok = |k: &str| {
        k.len() <= MAX_FOLD_CHARS
            && k.split_once(':').is_some_and(|(screen, peer)| {
                !screen.is_empty()
                    && screen.bytes().all(|b| b.is_ascii_lowercase())
                    && (peer == "me"
                        || (!peer.is_empty() && peer.bytes().all(|b| b.is_ascii_digit())))
            })
    };
    if keys.len() > MAX_FOLDED || !keys.iter().all(|k| ok(k)) {
        return Err(format!(
            "ui_folded 는 '{{화면}}:{{상대 id | me}}' {MAX_FOLDED}개까지"
        ));
    }
    Ok(keys.join(","))
}

/// 접은 절 열쇠 목록(없으면 빈 목록).
pub fn folded(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    Ok(settings::get(conn, FOLDED_KEY)?
        .map(|v| {
            v.split(',')
                .filter(|k| !k.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default())
}

/// '나' 타일(표시 이름, 아이콘) — 비었거나 틀린 값은 `None`.
pub fn me_tile(conn: &Connection) -> rusqlite::Result<(Option<String>, Option<String>)> {
    Ok((
        settings::get(conn, ME_ALIAS_KEY)?.and_then(|a| clean_alias(&a).ok().flatten()),
        settings::get(conn, ME_ICON_KEY)?.and_then(|i| clean_icon(&i).ok().flatten()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_effort_group_rules() {
        for ok in MODELS {
            assert!(valid_model(ok), "{ok}");
        }
        for bad in ["", "-x", "Sonnet", "claude-sonnet-5-5", "sonnet[1m]", "a;b"] {
            assert!(!valid_model(bad), "{bad}");
        }
        assert!(EFFORTS.iter().all(|e| valid_effort(e)));
        assert!(!valid_effort("HIGH") && !valid_effort(""));
        assert_eq!(clean_group("  개발팀 "), Ok(Some("개발팀".into())));
        assert_eq!(clean_group("   "), Ok(None));
        assert!(clean_group(&"가".repeat(33)).is_err());
        assert!(clean_group("개발\n팀").is_err() && clean_group("a\u{7}").is_err());
        assert_eq!(
            clean_group(&"가".repeat(32))
                .unwrap()
                .unwrap()
                .chars()
                .count(),
            32
        );
    }

    #[test]
    fn alias_title_icon_rules() {
        assert_eq!(clean_alias(" 세일즈 "), Ok(Some("세일즈".into())));
        assert!(clean_alias(&"가".repeat(33)).is_err() && clean_alias("a\tb").is_err());
        assert_eq!(clean_title(""), Ok(None));
        assert!(clean_title(&"가".repeat(100)).is_ok() && clean_title(&"가".repeat(101)).is_err());
        let png = format!("{ICON_PREFIX}AAAANSUhEUgAAAFAAAABQCAYAAACOEfKt=");
        assert_eq!(clean_icon(&png), Ok(Some(png.clone())));
        assert_eq!(clean_icon(""), Ok(None), "빈 값 = 지움");
        for bad in [
            "data:image/svg+xml;base64,PHN2Zz4=".to_string(),
            "data:image/png;base64,AAAA".to_string(),
            format!("{ICON_PREFIX}\"><script>"),
            format!("{ICON_PREFIX}{}", "A".repeat(MAX_ICON_CHARS)),
        ] {
            assert!(clean_icon(&bad).is_err(), "{bad:.60}");
        }
    }
}
