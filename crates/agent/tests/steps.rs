//! 처리경과(#1c A10) — stream-json 이벤트 → 단계 요약 · MCP 이름 묶기 · 받는 쪽 합침 · 보낸 쪽 진행에 단계(이름 정리 · 입력 없음).
mod common;
use an_agent::chat::Run;
use an_agent::runner::{RunInput, RunOutput, Session, count_step};
use an_core::{agents, chat};
use an_net::proto::{MAX_STEP_NAME, StepDetail};
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 실제 2.1.296 stream-json 모양(내용 블록마다 `assistant` 한 줄, 도구 결과는 `user`).
fn events() -> Vec<Value> {
    vec![
        json!({"type":"system","subtype":"init","session_id":"s","tools":["Read","Grep"]}),
        json!({"type":"assistant","message":{"content":[{"type":"thinking","thinking":"C:/secret/plan.txt 를 본다"}]}}),
        json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"file_path":"C:/secret/plan.txt"}}]}}),
        json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1","content":"비밀 본문"}]}}),
        json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"t2","name":"Grep","input":{"pattern":"token"}}]}}),
        json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"t3","name":"Read","input":{"file_path":"C:/secret/b.txt"}}]}}),
        json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","content":"x"},{"type":"tool_result","tool_use_id":"t3","content":"y"}]}}),
        json!({"type":"assistant","message":{"content":[{"type":"text","text":"답 본문"}]}}),
        json!({"type":"result","subtype":"success","is_error":false,"result":"답 본문","session_id":"s","num_turns":3}),
    ]
}

#[test]
fn stream_events_become_step_summary_without_inputs() {
    let mut d = StepDetail {
        turns: 1,
        ..StepDetail::default()
    };
    let changed: Vec<bool> = events().iter().map(|e| count_step(&mut d, e)).collect();
    assert_eq!(
        changed,
        [false, false, true, true, true, true, true, false, false],
        "도구 · 도구 결과만 단계"
    );
    assert_eq!(d.turns, 3, "도구 결과 user 2번 + 1");
    assert_eq!(
        d.tools.iter().map(|(k, v)| (k.as_str(), *v)).collect::<Vec<_>>(),
        [("Grep", 1), ("Read", 2)]
    );
    let wire = serde_json::to_string(&d).unwrap();
    for leak in ["secret", "비밀", "token", "답 본문"] {
        assert!(!wire.contains(leak), "요약에 입력 · 경로 · 글 없음: {wire}");
    }
    // 프롬프트 echo 등 도구 결과가 아닌 user 는 턴이 아니다.
    let echo = json!({"type":"user","message":{"content":[{"type":"text","text":"질문"}]}});
    assert!(!count_step(&mut d, &echo));
}

#[test]
fn mcp_tools_grouped_as_one_name() {
    let mut d = StepDetail::default();
    for name in ["mcp__azure__wit_get_work_item", "mcp__windbg__run_cmd", "Read"] {
        count_step(
            &mut d,
            &json!({"type":"assistant","message":{"content":[{"type":"tool_use","name":name,"input":{}}]}}),
        );
    }
    assert_eq!(
        d.tools.iter().map(|(k, v)| (k.as_str(), *v)).collect::<Vec<_>>(),
        [("MCP", 2), ("Read", 1)],
        "서버 · 도구 이름은 보내지 않는다"
    );
}

/// 실행기 대역 — `emits` 번 `gap` 마다 단계 요약을 내고(도구 이름 `names`) 답한다.
fn stepping(
    emits: u32,
    gap: Duration,
    names: &'static [&'static str],
) -> impl FnOnce(Arc<Mutex<Vec<RunInput>>>) -> Run {
    move |runs| {
        Arc::new(move |input: RunInput| {
            runs.lock().unwrap().push(input.clone());
            for i in 1..=emits {
                let mut d = StepDetail {
                    turns: i,
                    secs: i,
                    ..StepDetail::default()
                };
                for n in names {
                    d.tools.insert((*n).into(), i);
                }
                input.steps.emit(&d);
                std::thread::sleep(gap);
            }
            let (Session::New(id) | Session::Resume(id)) = &input.session;
            Ok(RunOutput {
                text: "답".into(),
                session: id.clone(),
                is_error: false,
                turns: Some(1),
                duration_ms: Some(1),
            })
        })
    }
}

/// 보낸 쪽이 그 글의 진행에서 본 서로 다른 단계 요약 — 답이 올 때까지 10ms 마다.
async fn watch_steps(a: &common::Side, b: &common::Side, text: &str) -> Vec<StepDetail> {
    agents::add(&common::db(b), "review", ".", None, "").unwrap();
    let (b_id, _) = common::pair(a, b).await;
    let (conv, sent) = a.chat.send(b_id, "review", None, text).await.unwrap();
    let mut seen: Vec<StepDetail> = Vec::new();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let m = chat::messages(&common::db(a), conv.id)
            .unwrap()
            .into_iter()
            .find(|m| m.id == sent.id)
            .unwrap();
        if let Some(d) = a.chat.progress_of(&m).and_then(|p| p.detail)
            && seen.last() != Some(&d)
        {
            seen.push(d);
        }
        if chat::messages(&common::db(a), conv.id).unwrap().len() >= 2 {
            return seen;
        }
        assert!(Instant::now() < until, "답이 오지 않았다");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn receiver_coalesces_steps_per_window() {
    let a = common::side("stc-a", Duration::ZERO).await;
    let b = common::side_with("stc-b", stepping(30, Duration::from_millis(50), &["Read"])).await;
    b.chat.set_step_every(Duration::from_millis(400));
    let seen = watch_steps(&a, &b, "합침").await;
    // 1.5초 동안 30번 바뀜 → 400ms 마다 마지막 값 하나(3 ~ 4번).
    assert!((2..=5).contains(&seen.len()), "{seen:?}");
    let turns: BTreeSet<u32> = seen.iter().map(|d| d.turns).collect();
    assert!(turns.len() == seen.len() && turns.iter().all(|t| *t > 1), "마지막 값으로 합침: {seen:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn sender_progress_shows_cleaned_steps() {
    let a = common::side("sts-a", Duration::ZERO).await;
    let b = common::side_with(
        "sts-b",
        stepping(
            8,
            Duration::from_millis(100),
            &["Read", "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", "bad\u{1b}[31mname"],
        ),
    )
    .await;
    b.chat.set_step_every(Duration::from_millis(200));
    let seen = watch_steps(&a, &b, "단계").await;
    let last = seen.last().expect("진행에 단계 요약");
    assert!(last.turns > 1 && last.secs > 0, "{last:?}");
    assert_eq!(last.tools.get("Read"), Some(&last.turns));
    assert!(
        last.tools
            .keys()
            .all(|k| k.chars().count() <= MAX_STEP_NAME && !k.contains('\u{1b}')),
        "받은 이름은 다시 거른다: {last:?}"
    );
}
