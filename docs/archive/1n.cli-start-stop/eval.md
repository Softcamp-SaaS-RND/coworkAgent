# eval.md — 1n.cli-start-stop 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: ed71e24c7c3efe0c17222401124dae8050c2589f

- [x] E1: A1·A2·A3·A5·A6·A7·A8·A9 — 워크스페이스 테스트 전부 통과(lifecycle: start · 다시 start · 모드 바꿈 · 부모 끝 · 동시 start · 시작 실패 8 · 오류 트레이 stop · 꺼진 stop · 보조 감시 즉시 끝 / watchdog StartFailed / 기존 CLI · 핸드오프)
  CHECK: cargo test --workspace
  EXPECT: /Running tests.lifecycle\.rs/
  EVIDENCE: Running tests\lifecycle.rs (target\debug\deps\lifecycle-4ce8110e2db35406.exe)

- [x] E2: A9 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A4·A5 — 실제 실행 파일 수명 시험만 따로(start · open · stop · 부모 · 동시 · 시작 실패)
  CHECK: cargo test -p coworkagent-app --test lifecycle
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.61s

- [x] E4: A9 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E5: A9 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E6: A9 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E7: A8 — 기존 CLI · 사용법 문구 회귀 없음
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.46s

- [x] E8: A1·A2·A4·A6·A8 — 이 PC 실제 확인(디버그 실행 파일 · 임시 `COWORKAGENT_DATA_DIR`): 인자 없는 실행 = 트레이 아이콘 보임 → `start` 로 트레이 사라짐 ·
  EVIDENCE: 2026-10-17 Claude(이 세션) · 이 PC Windows 11 Pro 26200 · 디버그 `target\debug\coworkAgent.exe`(`861e8b2`, 라운드 1 · 2 고침 뒤 `e4b1cbd` · `784a710` 에서 같은 순서 다시 — 결과 같음) + 임시 `COWORKAGENT_DATA_DIR` — 인자 없는 실행 mode=tray · 프로세스 2 · 알림 영역 UIA `SystemTray.NormalButton:coworkAgent` 있음 → `start` rc=0 "시작함" · mode=headless · 같은 버튼 없음 · 프로세스 2 → `open` rc=0 "열었음"(Edge 에 coworkAgent 탭) → 인자 없는 두 번째 실행 0 · 모드 · 프로세스 그대로 → `stop` rc=0 "종료함" · 프로세스 0 · instance.json 없음 / 부모 셸(별도 powershell) 끝 → 프로세스 0 / 꺼진 stop "실행 중이 아님" / 실제 데이터 폴더 16개 파일 전후 같음(실제 인스턴스는 떠 있지 않았음). 기록 [stage2](work/stage2.md)


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-17 · `/codex:adversarial-review --base 0138b00 --scope branch` + focus(intent A1 ~ A9 ↔ plan 해석 · E1 ~ E8 ↔ 구현, 잠금 경쟁 · 엉뚱한 PID ·
  트레이 · 감시 둘 · 릴리스 시험 변수 · Windows / Unix) · 세션 codex-review · HEAD `ec327a4`(요청과 일치) · job `review-mvbmx0pr-57g0q9` — Verdict
  needs-attention(high 2 · medium 2). base `0138b00` = stage 1 직전(그 앞 masterPlan 압축 커밋은 이 Task 와 무관해 뺐다).

원문:

```text
- [high] 강제 종료가 PID 소유권을 증명하지 못하며 앱을 남길 수 있음 (crates/app/src/control.rs:227-239)
  `stop`은 잠금 확인과 PID 파일 읽기·종료를 별개로 수행한다. 잠금 보유자가 사이에 종료되거나 PID가 재사용되면 `TerminateProcess`/`SIGKILL`이
  무관한 프로세스를 겨냥할 수 있다. 반대로 supervisor를 먼저 죽인 순간 자식이 아직 `instance.lock`을 잡기 전이면 앱 PID 종료를 건너뛰고
  `wait_released`가 즉시 성공한 뒤 자식만 살아남을 수 있다. 이는 A5의 "앱·감시 모두 종료"를 깨고 데이터 손상 가능성도 만든다.
  Recommendation: PID 파일 단독 종료를 제거하고 supervisor가 소유한 Windows Job Object/Unix process group 전체를 종료·대기한다. 불가피하면 PID와
  process creation identity·실행 파일을 함께 기록하고 종료 직전 재검증하며, stop/start 전용 잠금과 안정된 quiescence 확인을 추가한다.
- [high] MSVC abort와 시작 실패 코드 3을 구분하지 못함 (crates/app/src/watchdog.rs:56-59)
  watchdog는 자식의 모든 종료 코드 3을 `StartFailed`로 해석한다. 문서에도 MSVC `abort()`가 같은 코드라는 미조치 위험이 기록돼 있다. 정상 가동 뒤
  native abort가 발생해도 재시작하지 않고 오류 트레이 또는 headless 종료로 끝나 기존 감시 복구 계약이 깨지며, 과거 사유 파일을 시작 실패 원인으로
  표시할 수도 있다.
  Recommendation: raw exit code만으로 시작 실패를 판정하지 않는다. 현재 자식 PID/세대에 묶인 원자적 startup-failure marker나 별도 IPC를 함께
  확인하고, marker 없는 코드 3은 일반 crash로 재시작한다.
- [medium] stop의 10초 상한이 실제로는 15초 이상임 (crates/app/src/control.rs:216-263)
  응답 없는 경로는 먼저 5초를 기다린 뒤 강제 종료하고, 이후 `wait_released`가 새 10초 deadline을 만든다. 각 `probe`도 최대 1초 걸려 전체 지연은
  더 길다. 따라서 A5의 "최대 10초"를 넘긴 뒤에도 성공할 수 있다. `start` 역시 30초 deadline 직전에 별도 10초 `quit`을 시작할 수 있다. lifecycle
  시험은 이 경계값을 검사하지 않는다.
  Recommendation: 명령 진입 시 하나의 절대 deadline을 만들고 probe·quit·강제 종료·잠금 대기에 남은 시간만 전달한다. 5/10/30초 및 ±1 경계를
  fake clock 또는 짧은 시험 상수로 검증한다.
- [medium] 구버전 실행 인스턴스를 start/stop으로 종료할 수 없음 (crates/app/src/control.rs:244-249)
  `probe`는 `mode`가 없는 예전 `/api/status`를 `tray`로 간주하지만, `quit`은 예전 서버에 없는 `/api/quit`의 404나 전송 실패를 무시하고 잠금 해제만
  기다린다. 응답 가능한 구버전은 unresponsive 강제 종료 분기로도 가지 않아 새 `start`와 `stop`이 timeout으로 끝난다. 실행 중 업그레이드·별도 경로
  배포 시 제어 불능 상태가 된다.
  Recommendation: `/api/quit`이 204가 아니면 capability/version을 판별해 안전한 legacy 종료 경로로 전환하거나 명시적 비호환 오류를 즉시 반환한다.
  직전 릴리스 router를 띄운 version-skew 통합 시험을 추가한다.
```

판단 · 조치:

- **[high] 강제 종료 PID 소유권 · 남는 앱** — 판단: **사실**(두 갈래 모두). **고침** — 새 `proc::kill_tree(pid)`: Windows 는 핸들을 연 채
  `QueryFullProcessImageNameW` 로 이 실행 파일과 이름이 같을 때만 끊고(핸들을 쥐는 동안 PID 는 재사용되지 않는다), 감시를 끊은 뒤 Toolhelp 부모 PID 로
  자식(같은 이름)도 끊는다 — 잠금을 잡기 전의 앱도 끊긴다. Unix 는 감시가 프로세스 그룹 우두머리면 그룹째 `SIGKILL`. `stop` 은 끊은 뒤 잠금이 300 ms
  풀린 채 머무는지 보고 다시 잡혔으면 되풀이(상한 안). Job Object 로 바꾸기 · 생성 시각 기록은 하지 않았다(아래 미조치). 시험
  `proc::kill_tree_spares_other_programs`(이름 다른 프로세스는 남김) · lifecycle `error_tray_is_single_and_stoppable`(강제 종료 길).
- **[high] 코드 3 ↔ MSVC `abort()`** — 판단: **사실**(stage 1 미조치로 적었던 것). **고침** — 코드 값은 plan 대로 3, `watchdog::spawn_app` 이 자식을 띄우기
  직전 시각을 잡고 사유 파일이 그 뒤에 쓰였을 때만 3 을 넘긴다(`tray::start_error_since`) — 사유 없는 3 은 -1(비정상 종료)로 다시 띄운다. 시험: 시작 실패
  길은 lifecycle `start_failure_exits_8_with_reason`. 사유 없는 3(실제 `abort`) 시험은 없다(아래 미조치).
- **[medium] stop 상한 15초+** — 판단: **사실**. **고침** — `stop` 진입 때 상한 하나(`STOP_LIMIT` 10초)를 만들어 응답 기다림 · `quit` · 강제 종료 뒤
  잠금 대기가 나눠 쓴다. `start` 안의 `quit` 은 `min(start 상한, 지금 + 10초)`. probe 한 번(최대 1초)만큼 넘칠 수 있다. 경계 ±1 시험은 없다(아래 미조치).
- **[medium] 이전 버전 인스턴스** — 판단: **사실**. **고침** — `/api/quit` 이 404 면 기다리지 않고 4 "떠 있는 앱이 종료 명령이 없는 이전 버전이다 — 트레이
  \"종료\" 로 끈 뒤 다시"(`start` · `stop` 공통). 시험 lifecycle `old_version_fails_fast`(종료 경로 없는 `router_with` 서버 — 3초 안 · 4 · 문구). 사용자
  가이드 14.1 · 16장.

## 적대적 평가 — 라운드 2/2

- 2026-10-17 · `/codex:adversarial-review --base 0138b00 --scope branch` + 라운드 1 과 같은 focus(+ 라운드 1 고침 확인) · 세션 codex-review(라운드 1 과 같음) ·
  HEAD `7122bea`(요청과 일치) · job `review-mvbng0uy-x7p9r2` — Verdict needs-attention(high 3 · medium 1).

원문:

```text
- [high] 강제 종료가 여전히 다른 PID 또는 Unix 프로세스 그룹을 죽일 수 있음 (crates/app/src/proc.rs:157-229)
  Windows 검증은 전체 실행 경로나 생성 시각이 아니라 basename만 비교한다. 따라서 PID 재사용 뒤 다른 경로의 `coworkAgent.exe`를 종료할 수 있다.
  Unix 구현은 코드 주석대로 소유권 검증 없이 재사용된 PID 또는 `-pid` 프로세스 그룹 전체에 SIGKILL을 보낸다. 잠금 확인과 PID 조회·종료도
  원자적이지 않다. 라운드 1 시험은 이름이 다른 `ping`만 보호하며 같은 basename·Unix 경로를 검증하지 않는다.
  Recommendation: Windows는 named Job Object 또는 canonical executable path와 process creation time을 결합해 재검증한다. macOS는 `proc_pidpath`와
  생성 identity를 확인한 뒤 신호를 보내고, 잠금 소유권 확인부터 종료까지 안정된 identity로 묶는다. 같은 basename의 다른 실행 파일과 재사용 PID
  시험을 추가한다.
- [high] 전역 사유 파일 mtime은 현재 자식의 시작 실패를 증명하지 못함 (crates/app/src/watchdog.rs:74-84)
  `spawn_app`은 종료 코드 3과 전역 사유 파일의 `modified >= spawned`만으로 `StartFailed`를 판정한다. 같은 데이터 루트의 보조 실행이
  `fail_unlocked`에서 사유 파일을 쓰거나 미래 시각의 stale 파일이 있으면, 정상 가동 뒤 현재 자식의 MSVC abort도 시작 실패로 오인해 watchdog
  재시작을 중단한다. 반대로 시계 역행·mtime 문제는 실제 시작 실패를 crash loop로 바꿀 수 있다. 추가된 시험은 실제 시작 실패만 다루며 사유 없는
  코드 3 경로는 검증하지 않는다.
  Recommendation: spawn마다 nonce를 생성해 자식에게 전달하고, PID·nonce가 포함된 원자적 marker 또는 IPC로 현재 세대의 실패만 인정한다.
  stale/concurrent marker와 사유 없는 코드 3 회귀 시험을 추가한다.
- [high] start와 stop 전환이 직렬화되지 않아 stop 성공 뒤 새 인스턴스가 남을 수 있음 (crates/app/src/control.rs:117-176)
  `launch`와 `stop`은 공통 제어 잠금 없이 `probe`·`held` 관찰 결과로 각각 행동한다. 꺼진 상태에서 stop이 잠금 없음으로 0을 반환하는 동안
  concurrent start가 `spawn_detached` 직전이면 이후 앱이 뜬다. 트레이 전환 중에는 start와 stop이 함께 quit한 뒤 start가 잠금 해제를 보고 다시
  headless 인스턴스를 띄울 수 있다. 결과적으로 stop이 성공해도 앱·감시가 남아 A5와 실행 파일 교체 용도를 깨뜨린다. lifecycle 시험도 start끼리만
  경쟁시키며 start/stop/open/인자 없는 실행 혼합 경쟁은 없다.
  Recommendation: 전체 start/open/stop 상태 전환을 `control.lock` 또는 desired-state generation으로 직렬화하고 supervisor도 stop tombstone을
  확인하게 한다. barrier 기반 동시 start+stop, tray→headless+stop, 인자 없는 실행+stop 시험에서 stop 반환 시 두 잠금과 프로세스가 모두 없음을
  단정한다.
- [medium] stop의 단일 deadline이 실제 blocking 구간 전체를 제한하지 않음 (crates/app/src/control.rs:230-247)
  `wait_released`가 deadline 직전에 성공해도 이후 `SETTLE` 300ms를 무조건 자고 성공하므로 10초를 넘길 수 있다. 또한 `kill_tree`는 Windows에서
  PID마다 최대 1초를 기다리고 probe도 고정 1초 timeout을 사용하지만 남은 시간으로 제한되지 않는다. eval도 probe 한 번만큼 초과 가능하고 경계
  시험이 없다고 인정한다. 따라서 A5의 최대 10초 계약은 아직 보장되지 않는다.
  Recommendation: 모든 probe·kill wait·settle에 절대 deadline의 remaining duration을 전달하고, settle 뒤에도 deadline을 재검사한다. 10초 직전·정확히
  10초·직후 경계를 fake clock 또는 짧은 시험 상수로 검증한다.
```

판단 · 조치(상한 도달 — 고친 뒤 재리뷰하지 않는다):

- **[high] 강제 종료 PID · Unix 그룹** — 판단: **사실(남은 위험)**. 고치지 않음 — 미조치 표 1 · 2.
- **[high] 사유 파일 mtime** — 판단: **사실**. **고침** — 사유 파일 끝에 쓴 프로세스의 `PID: n` 줄, `spawn_app` 은 자식 핸들을 쥔 채(그 사이 PID 재사용 없음)
  사유 파일이 그 자식의 PID 를 담을 때만 3 을 넘긴다(`tray::start_error_by`) — 다른 실행이 쓴 사유 · 시계 · mtime 과 무관. 사유 없는 3 시험은 없다(미조치 표 5).
- **[high] start · stop 직렬화 없음** — 판단: **설계 판단 — 사용자 확인 필요**. 고치지 않음 — 미조치 표 3.
- **[medium] 상한이 모든 대기를 묶지 않음** — 판단: **사실**. **일부 고침** — 강제 종료 뒤 확인 대기(300 ms)를 남은 시간으로 자른다. probe(1초) · `kill_tree` 대기
  (PID 당 1초)는 그대로 — 미조치 표 4.

재검증: `cargo test --workspace` ok 73건 · 실패 0 · clippy Finished · gate 원장(위, 측정 커밋 갱신).

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| R1 · R2 [high] 강제 종료가 PID 소유권을 증명하지 못함 — Job Object 또는 전체 경로 + 생성 시각으로 재검증 | 사실(남은 위험) | Windows 는 핸들을 연 채 이름이 같을 때만 끊는다 — 남는 창은 잠금 확인 ~ `OpenProcess` 사이(ms)에 보유자가 끝나고 그 PID 를 **같은 이름**의 다른 프로세스가 받은 경우뿐. 전체 경로 비교는 다른 폴더의 사본으로 떠 있는 앱을 `stop` 하지 못하게 한다(같은 데이터 루트 잠금 보유자를 끄는 것이 `stop` 의 뜻). Job Object 는 감시의 자식 생성 방식을 바꾸는 것 — plan 범위 밖 |
| R2 [high] Unix 는 소유권 확인 없이 PID · `-pid` 그룹에 SIGKILL | 사실(남은 위험) | 그룹째는 감시가 그룹 우두머리일 때만(`getpgid(pid) == pid`). 이 PC 는 Unix 대상 컴파일 · 실행 불가 — macOS 실기 확인(원장 19 인수)과 함께 `proc_pidpath` 확인을 넣는다 |
| R2 [high] start · open · stop 을 공통 잠금으로 직렬화하지 않아 stop 성공 뒤 동시 start 의 앱이 남음 | 설계 판단 — 사용자 확인 필요 | 동시 명령의 결과가 직렬 순서 하나("stop → start")와 같다 — 그때 `start` 는 "시작함" 을 돌려준다. 어느 경로로도 앱 · 감시가 둘 이상 되지는 않는다(A6). 실행 파일 교체 중 부모 도구가 `start` 를 부르는 경우까지 막으려면 `control.lock` · stop 표식 — 정책 결정 |
| R2 [medium] probe(1초) · `kill_tree` 대기(PID 당 1초)가 남은 시간으로 묶이지 않음 | 사실 | 최악 약 2초 넘친다(10초 → 약 12초에 6). 확인 대기는 고침. 둘을 남은 시간으로 자르는 것은 작은 변경이지만 상한 도달로 다음 Task 에 |
| R1 · R2 시험 공백 — 상한 경계 ±1 · 사유 없는 코드 3(`abort`) · start / stop / 인자 없는 실행 혼합 경쟁 · 같은 이름 다른 실행 파일 | 사실(시험 공백) | 가짜 시계 · 시험용 abort 훅은 제품 코드에 시험 스위치를 늘린다(eval-protocol 6 "릴리스 잔존 금지"). 코드 사실 확인으로 갈음 |
