//! 잠금 — 한 번 오염돼도(가드를 쥔 채 panic) 안의 값을 계속 쓴다(1a A14). 오염 원인은 이미 panic 으로 로그에 남았다.
use std::sync::{Mutex, MutexGuard, PoisonError};

pub fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e: PoisonError<_>| {
        log::warn!("오염된 잠금을 이어 쓴다");
        e.into_inner()
    })
}
