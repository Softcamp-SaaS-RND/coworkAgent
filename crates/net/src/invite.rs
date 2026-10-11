//! 페어링 초대 — 받는 쪽이 메신저로 보내는 텍스트(P10). 사람이 읽는 두 줄 + 기계가 읽는 한 줄 토큰
//! `coworkagent-invite:v1/{주소}/{초대 코드}/{지문 hex}`. 토큰에는 공백이 없어 메신저가 줄을 바꿔도 남는다.
use std::net::SocketAddr;

const MARK: &str = "coworkagent-invite:v1/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invite {
    pub addr: SocketAddr,
    /// 초대 코드(정규화 전 표기 그대로 — 비교는 받는 쪽이 정규화).
    pub code: String,
    /// 받는 쪽 지문 — hex 32자(공백 없음, 대문자).
    pub fingerprint: String,
}

/// 지문 표기(`7F3A 19C2 …`)를 hex 대문자 32자로. 형식이 아니면 `None`.
pub fn normalize_fingerprint(s: &str) -> Option<String> {
    let hex: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != ':')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    (hex.len() == 32 && hex.bytes().all(|c| c.is_ascii_hexdigit())).then_some(hex)
}

impl Invite {
    /// 메신저로 보낼 텍스트.
    pub fn to_text(&self, name: &str, host: &str) -> String {
        format!(
            "coworkAgent 페어링 초대 — {name} · {host}\n\
             5분 안에 coworkAgent 웹UI \"초대 붙여넣기\" 에 이 메시지를 통째로 붙여 넣으세요.\n\
             {MARK}{}/{}/{}",
            self.addr, self.code, self.fingerprint
        )
    }

    /// 붙여 넣은 텍스트 어디에 있든 토큰 하나를 찾아 읽는다. 없거나 형식이 틀리면 `None`.
    pub fn parse(text: &str) -> Option<Self> {
        let token = text.split(MARK).nth(1)?.split_whitespace().next()?;
        let mut parts = token.split('/');
        let (addr, code, fp) = (parts.next()?, parts.next()?, parts.next()?);
        if parts.next().is_some() || code.is_empty() {
            return None;
        }
        Some(Self {
            addr: addr.parse().ok()?,
            code: code.to_string(),
            fingerprint: normalize_fingerprint(fp)?,
        })
    }
}
