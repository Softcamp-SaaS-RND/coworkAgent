# Windows — crates.io TLS 인증서 폐기 확인 실패

> 환경 함정 노트. 이 리포에서 처음 `cargo build` 하기 전에 읽는다.

## 증상

`cargo build` · `cargo search` 가 crates.io 접속에서 실패한다(2026-10-11, 개발 PC 실측):

```text
[35] SSL connect error (schannel: next InitializeSecurityContext failed: CRYPT_E_NO_REVOCATION_CHECK (0x80092012))
```

## 원인

Windows schannel 이 crates.io 인증서의 폐기 여부를 확인하지 못한다 — 폐기 확인 서버에 닿지 않는 사내망에서 흔하다.
같은 PC 에서 api.github.com 은 정상이었다(대상 호스트마다 다르다).

## 조치

폐기 확인만 끈다 — 인증서 체인 검증은 그대로다(`http.check-revoke` 는 Windows 전용 설정). 개발자별로 둘 중 하나:

- 셸 세션 환경변수 `CARGO_HTTP_CHECK_REVOKE=false`
- 사용자 설정 `~/.cargo/config.toml`

```toml
[http]
check-revoke = false
```

리포의 `.cargo/config.toml` 에는 넣지 않았다 — 보안 설정 완화를 모든 개발자에게 강제하지 않기 위해서다(바꾸려면
사용자 결정). 의존을 한 번 받은 뒤에는 `Cargo.lock` 과 로컬 레지스트리 캐시로 빌드되어 네트워크가 필요 없다.
curl 은 `--ssl-no-revoke`.

## 확인

```bash
CARGO_HTTP_CHECK_REVOKE=false cargo search axum --limit 1   # axum = "0.8.9" (2026-10-11)
```
