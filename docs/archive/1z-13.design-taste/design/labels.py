"""1z-13 문구 그대로 — index.html 의 한글 글(글 마디 · aria-label · title · placeholder)이 기준 커밋(인자)과 같다."""
import collections
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[4]  # docs/{tasks|archive}/{key}/design/labels.py
REL = "crates/app/web/index.html"


def labels(html):
    out = collections.Counter()
    for m in re.finditer(r'(?:aria-label|title|placeholder)="([^"]*[가-힣][^"]*)"', html):
        out[m.group(1)] += 1
    for m in re.finditer(r">([^<>]*[가-힣][^<>]*)<", html):
        out[" ".join(m.group(1).split())] += 1
    return out


old = subprocess.run(["git", "-C", str(ROOT), "show", f"{sys.argv[1]}:{REL}"], capture_output=True, check=True).stdout.decode("utf-8")
base, now = labels(old), labels((ROOT / REL).read_text(encoding="utf-8"))
for k in base - now:
    print(f"missing {k}", file=sys.stderr)
for k in now - base:
    print(f"added {k}", file=sys.stderr)
print(f"labels n={sum(base.values())} missing={sum((base - now).values())} added={sum((now - base).values())}")
