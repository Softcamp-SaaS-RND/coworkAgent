# intent — 30.auto-update

승인: 2026-10-21

## 왜

사용자 요청(대화 2026-10-21) 원문:
"https://github.com/Softcamp-SaaS-RND/coworkAgent 에 windows 용 release 배포.
또한, coworkAgent.exe 는 실행 시 위 release 에 최신 버전이 있을 경우 해당 버전으로 자동 업데이트 되도록 실행 시 업데이트 workflow 추가."

질문 답(대화 2026-10-21):
- 검증: "인증서 서명은 `D:\dev\git\coworkAgent\target\release\coworkAgent.exe` 에 직접 서명한 exe 가 있음. 해당 exe 를 github release 에 등록." →
  다시 물음 "Authenticode + 서명자 확인".
- 버전: "v1.1.0 버전." → exe 속성 버전이 1.1.1 이라 다시 물음 "v1.1.1 정식(Latest)".
- workflow 범위: "실행 시 업데이트 흐름만".
- 다음 버전 배포: "다음에도 서명된 exe 는 이전과 같은 로컬 경로에 등록. 해당 파일 서명 확인해서 github 에 release 등록."

— 출처: 대화, masterPlan D12(업데이트 파일 서명 검증) · D36(배포처 · 코드 서명 · 자동 업데이트 TODO) · S6 · D13(감시 프로세스), #1y(GitHub 릴리스 자동 업데이트 —
만들었다가 2026-10-20 사용자 지시로 걷어냄, 코드는 `8610de5^` 에 2 라운드 평가까지 끝난 채 남음).

### 코드 사실 (2026-10-21 확인)

| 사실 | 위치 |
| --- | --- |
| 자동 업데이트 코드 없음 — #1y 종결 커밋이 `update.rs` · 감시 연결 · `ureq` · 시험을 지움 | `8610de5` |
| 걷어내기 직전 업데이터 = 정식 릴리스 `releases/latest` 302 · 엄격히 큰 버전 · PE 꼴 확인 · `.old` 교체 · 뜨지 못하면 되돌림 · 서명 확인 없음 | `8610de5^:crates/app/src/update.rs` |
| 배포처 저장소 `Softcamp-SaaS-RND/coworkAgent` = PUBLIC, 릴리스 0개(요청 시점) | `gh repo view` · `gh release list` |
| 서명된 exe — Authenticode Valid, 서명자 `CN="SOFTCAMP CO.,LTD."`(DigiCert Trusted G4 Code Signing), 타임스탬프 DigiCert, 속성 버전 1.1.1 | `Get-AuthenticodeSignature` · `target/release/coworkAgent.exe` |
| 크레이트 버전 1.1.1 | `crates/*/Cargo.toml` |

## 범위

- 서명된 1.1.1 exe 를 `Softcamp-SaaS-RND/coworkAgent` 정식 릴리스 `v1.1.1`(Latest)로 등록(이 exe 에는 업데이터가 없다).
- #1y 업데이터 되살리기(감시 프로세스 · 트레이 모드 · 릴리스 빌드 · Windows) — 배포처 = `https://github.com/Softcamp-SaaS-RND/coworkAgent/releases`.
- 받은 exe 를 바꾸기 전에 Authenticode 확인(유효 · 신뢰 루트 · 코드 서명 · 폐기 확인) + 서명 인증서 이름(CN) `SOFTCAMP CO.,LTD.` — 아니면 바꾸지 않음.
- 버전 1.2.0(업데이터가 든 첫 버전) — 빌드 → 선생님이 `target/release/coworkAgent.exe` 에 서명 → 서명 · 버전 확인 → 정식 릴리스 등록.
- 매뉴얼 · README · PRODUCT · masterPlan(D12 · D36 · S6) 반영, 릴리스 절차(개발자 가이드 2.4).

## 제외

- GitHub Actions · 릴리스 파이프라인(질문 답 — 실행 시 흐름만), macOS 자동 업데이트(자산 없음 · Phase 4).
- 업데이트 되돌리기 UI · 상대 버전 차이 표시(D36 ④ 일부) · 주기 확인(실행 시점만).
- 서명 자체(선생님이 회사 인증서로 — 개인키는 이 저장소 · 개발 도구에 없다).

## 성공 기준

- A1: `v1.1.1` 정식 릴리스(Latest)에 서명된 `coworkAgent.exe` 가 있고, `releases/latest` 가 `v1.1.1` 로 넘기며 받은 파일 SHA-256 = 등록한 파일(실측).
- A2: 트레이 모드 감시가 앱을 띄우기 전 `Softcamp-SaaS-RND/coworkAgent` 의 최신 정식 릴리스를 확인해 더 새 버전이면 받아 바꾸고 새 버전으로 뜬다 — 실패는 지금 버전(시험 · 검사).
- A3: 받은 exe 는 Authenticode 가 유효하고 서명자 CN 이 `SOFTCAMP CO.,LTD.` 일 때만 바꾼다 — 서명 없음 · 고친 사본 · 다른 서명자 이름은 바꾸지 않고 `.new` 도 남기지 않는다(시험 · 실제 릴리스 실측).
- A4: 1.2.0 서명 릴리스를 Latest 로 등록하고, 업데이터가 든 낮은 버전 빌드가 실제로 그것으로 바뀌어 뜬다(실측).
- A5: 릴리스 절차(빌드 → 서명 → 확인 → 등록)와 자동 업데이트 안내가 개발자 가이드 · 사용자 가이드 · README 에 있다(검사).
- A6: 회귀 없음 — PROJECT.md 검증 명령 PASS.

## 결정

1. 자동 업데이트를 되살린다 — #1y 개정("git release 에 신규파일 등록 시 자동 배포되는 기능은 삭제", 2026-10-20) **재개정**(이번 요청). 코드는 `8610de5^` 를 그대로 되살리고
   배포처 · 서명 확인만 바꾼다(2 라운드 평가를 거친 되돌리기 · 15초 지켜보기 · `update-skip` 그대로).
2. 배포처 = `https://github.com/Softcamp-SaaS-RND/coworkAgent/releases`(D36 "Azure DevOps 이관" 을 자동 업데이트 · 릴리스에 한해 개정, 파이프라인은 TODO 그대로).
3. 검증 = **회사 인증서 Authenticode + 서명자 CN `SOFTCAMP CO.,LTD.`**(질문 답). D12 "앱 자체 ed25519" 를 개정 — #1y 결정 7 개정(사용자 "회사 인증서를 사용해야") 그대로의 방향.
   서명자 확인 = 인증서 이름만(지문 고정 아님 — 인증서 갱신 때 업데이트가 끊기지 않게). 폐기 확인 = 체인 전체(루트 제외) — 확인할 수 없으면 바꾸지 않는다.
   기각: HTTPS + PE 꼴만(#1y — 저장소를 빼앗기면 모든 PC).
4. 확인 시점 · 주체 · 교체 · 적용 — #1y 결정 6 · 8 · 17 · 18 · 19 그대로(실행 시점 · 감시가 앱 전에 · `.old` 이름 바꾸기 · 묻지 않고 · `ureq` + OS 인증서 저장소).
5. 첫 등록 = 서명된 1.1.1 exe → `v1.1.1` 정식(질문 답 — 요청의 "v1.1.0" 은 exe 속성 버전과 달라 다시 물어 바꿈). 업데이터가 든 첫 버전 = 1.2.0(1.1.1 사용자는 한 번 손으로 설치).
6. 서명 흐름 — 빌드 산출 `target/release/coworkAgent.exe` 에 선생님이 서명 → 내가 Authenticode · 서명자 · 속성 버전을 확인하고 등록(질문 답). 태그 = 그 코드가 든 origin/main SHA.
7. Task 번호 = Phase 3(배포 · 통합) 첫 Task `30`.
8. 승인 절차: 요청 · 질문 답을 intent · plan 승인으로 갈음(#1z-2 결정 9 와 같은 방식).

## 미해소

- 없음
