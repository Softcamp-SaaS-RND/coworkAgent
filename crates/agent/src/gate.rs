//! 실행 문 — PC 전체 에이전트 실행(`claude -p` · `codex exec`) 동시 실행 상한. 넘는 실행은 거절하지 않고 닿은 순서(FIFO)로 기다린다.
//! 대화 안 직렬은 대화 작업이 지키고, 이 문은 **대화 사이**를 묶는다.
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub struct Gate {
    inner: Mutex<Inner>,
}

struct Inner {
    cap: usize,
    /// 지금 실행 중인 메시지 id(닿은 순서).
    running: Vec<i64>,
    /// 기다리는 메시지 id 와 자리가 나면 깨울 신호.
    waiting: VecDeque<(i64, oneshot::Sender<()>)>,
}

/// 문 안의 자리 — 버리면(실행 끝 · 실패 · panic · 기다리다 취소) 자리를 놓고 다음 사람을 들인다.
pub struct Ticket {
    gate: Arc<Gate>,
    id: i64,
    admit: Option<oneshot::Receiver<()>>,
}

impl Ticket {
    /// 자리가 날 때까지 기다린다(바로 들어온 티켓은 곧바로 끝난다).
    pub async fn ready(&mut self) {
        if let Some(rx) = self.admit.take() {
            // 보낸 쪽(문)은 들일 때만 보내고 버리지 않는다 — 오류는 문이 사라졌을 때뿐이라 그대로 지나간다.
            let _ = rx.await;
        }
    }
}

impl Drop for Ticket {
    fn drop(&mut self) {
        let mut g = an_core::sync::lock(&self.gate.inner);
        g.running.retain(|&r| r != self.id);
        g.waiting.retain(|(w, _)| *w != self.id);
        g.pump();
    }
}

impl Inner {
    /// 자리가 있는 만큼 줄 앞에서 들인다.
    fn pump(&mut self) {
        while self.running.len() < self.cap {
            let Some((id, tx)) = self.waiting.pop_front() else {
                break;
            };
            self.running.push(id);
            let _ = tx.send(());
        }
    }
}

impl Gate {
    pub fn new(cap: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(Inner {
                cap: cap.max(1),
                running: Vec::new(),
                waiting: VecDeque::new(),
            }),
        })
    }

    /// 문에 닿는다. 돌려주는 값: (티켓, 기다리면 `Some(앞 사람 수)`). 기다리는 티켓은 [`Ticket::ready`] 로 들어간다.
    pub fn enter(self: &Arc<Self>, id: i64) -> (Ticket, Option<usize>) {
        let mut g = an_core::sync::lock(&self.inner);
        if g.waiting.is_empty() && g.running.len() < g.cap {
            g.running.push(id);
            let t = Ticket {
                gate: self.clone(),
                id,
                admit: None,
            };
            return (t, None);
        }
        let ahead = g.waiting.len();
        let (tx, rx) = oneshot::channel();
        g.waiting.push_back((id, tx));
        let t = Ticket {
            gate: self.clone(),
            id,
            admit: Some(rx),
        };
        (t, Some(ahead))
    }

    /// 상한을 바꾼다(1 이상) — 늘면 그 자리에서 줄을 당긴다. 줄면 실행 중인 것은 끝까지 두고 새로 들이지 않는다.
    pub fn set_cap(&self, cap: usize) {
        let mut g = an_core::sync::lock(&self.inner);
        g.cap = cap.max(1);
        g.pump();
    }

    /// (상한, 실행 중 id, 기다리는 id) — 닿은 순서.
    pub fn snapshot(&self) -> (usize, Vec<i64>, Vec<i64>) {
        let g = an_core::sync::lock(&self.inner);
        (
            g.cap,
            g.running.clone(),
            g.waiting.iter().map(|(id, _)| *id).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Gate;
    use std::time::Duration;

    /// 잠깐 안에 끝나면 참.
    async fn admitted(t: &mut super::Ticket) -> bool {
        tokio::time::timeout(Duration::from_millis(50), t.ready())
            .await
            .is_ok()
    }

    #[tokio::test]
    async fn cap_limits_running() {
        let g = Gate::new(2);
        let (_a, wa) = g.enter(1);
        let (_b, wb) = g.enter(2);
        let (mut c, wc) = g.enter(3);
        assert_eq!((wa, wb, wc), (None, None, Some(0)));
        assert!(!admitted(&mut c).await);
        assert_eq!(g.snapshot(), (2, vec![1, 2], vec![3]));
    }

    #[tokio::test]
    async fn fifo_and_ahead() {
        let g = Gate::new(1);
        let (a, _) = g.enter(1);
        let (mut b, wb) = g.enter(2);
        let (mut c, wc) = g.enter(3);
        let (_d, wd) = g.enter(4);
        assert_eq!((wb, wc, wd), (Some(0), Some(1), Some(2)));
        drop(a);
        assert!(admitted(&mut b).await);
        assert!(!admitted(&mut c).await, "한 번에 한 사람");
        assert_eq!(g.snapshot(), (1, vec![2], vec![3, 4]));
    }

    #[tokio::test]
    async fn raising_cap_admits_now() {
        let g = Gate::new(1);
        let (_a, _) = g.enter(1);
        let (mut b, _) = g.enter(2);
        let (mut c, _) = g.enter(3);
        g.set_cap(2);
        assert!(admitted(&mut b).await);
        assert!(!admitted(&mut c).await);
        g.set_cap(0); // 1 로 올림 — 실행 중은 그대로
        assert_eq!(g.snapshot(), (1, vec![1, 2], vec![3]));
    }

    #[tokio::test]
    async fn dropped_ticket_releases_running_and_waiting() {
        let g = Gate::new(1);
        let (a, _) = g.enter(1);
        let (b, _) = g.enter(2);
        let (mut c, _) = g.enter(3);
        // 기다리던 b 가 취소 — 줄에서 빠진다.
        drop(b);
        assert_eq!(g.snapshot(), (1, vec![1], vec![3]));
        // 실행 중이던 a 가 실패로 끝남(드롭) — 다음 사람 c.
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _held = a;
            panic!("실행 실패");
        }));
        assert!(r.is_err());
        assert!(admitted(&mut c).await);
        assert_eq!(g.snapshot(), (1, vec![3], vec![]));
    }

    #[tokio::test]
    async fn new_arrival_waits_behind_queue() {
        let g = Gate::new(2);
        let (a, _) = g.enter(1);
        let (_b, _) = g.enter(2);
        let (mut c, _) = g.enter(3);
        drop(a); // c 가 들어간다
        assert!(admitted(&mut c).await);
        // 자리가 꽉 찼다 — 새로 온 d 는 줄 끝.
        let (_d, wd) = g.enter(4);
        assert_eq!(wd, Some(0));
        assert_eq!(g.snapshot(), (2, vec![2, 3], vec![4]));
    }
}
