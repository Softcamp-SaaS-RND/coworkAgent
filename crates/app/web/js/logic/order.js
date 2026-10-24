// 순서 — 위치 정렬 · 옮기기 · 적용.

/// 손으로 놓은 순서 — `pos` 작은 것 먼저, 같으면 id(서버 `peers::list` 와 같은 규칙). 새 배열.
export function sortByPos(items) {
  return [...items].sort((a, b) => (a.pos ?? 0) - (b.pos ?? 0) || a.id - b.id);
}

/// 전체 순서(`order` — id 목록)에서 `id` 를 빼 `target` 바로 앞(`after` 면 뒤)에 끼운다 — 걸러 보인 목록에서 옮겨도 그 앞뒤가 다른 화면에도
/// 같다. 같은 id 거나 `target` 이 없으면 그대로(새 배열).
export function moveInOrder(order, id, target, after = false) {
  if (id === target || !order.includes(id) || !order.includes(target)) return [...order];
  const rest = order.filter((x) => x !== id);
  const i = rest.indexOf(target) + (after ? 1 : 0);
  return [...rest.slice(0, i), id, ...rest.slice(i)];
}

/// 늦게 온 목록 응답에 덮어 씌울 순서 — 저장 상태 `saved = {last, busy, stamp}`, `stamp` = 그 읽기를 시작할 때의 표. 저장 중이거나
/// 읽는 사이 옮김 · 저장이 있었으면 마지막으로 놓은 순서, 아니면 `null`(받은 그대로).
export function orderToApply(saved, stamp) {
  if (!saved?.last) return null;
  return saved.busy || saved.stamp !== stamp ? saved.last : null;
}

/// 새 순서를 화면 값에 바로 — `ids` 의 자리(0 ..)를 `pos` 로, 빠진 항목은 제 `pos` 그대로(서버 `PUT /api/order` 와 같은 규칙). 새 배열.
export function withOrder(items, ids) {
  const at = new Map(ids.map((id, i) => [id, i]));
  return items.map((x) => (at.has(x.id) ? { ...x, pos: at.get(x.id) } : x));
}

/// 옮긴 뒤 화면 읽기 프로그램 알림 — "{이름} · n번째로 옮김"(1 부터).
export function movedText(name, index) {
  return `${name} · ${index + 1}번째로 옮김`;
}
