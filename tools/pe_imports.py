#!/usr/bin/env python
"""Windows 실행 파일(PE32+)의 import DLL 목록 · `DependentLoadFlags` 를 읽는다 — 표준 라이브러리만.

    python tools/pe_imports.py target/release/coworkAgent.exe

마지막 줄: `imports=N vcruntime=V dependent_load_flag=0xF` — V = 이름이 `vcruntime` 으로 시작하는 DLL 수(1a A13, DLL 심기 방어).
"""

import struct
import sys
from pathlib import Path


def sections(b: bytes, pe: int):
    n = struct.unpack_from("<H", b, pe + 6)[0]
    opt_size = struct.unpack_from("<H", b, pe + 20)[0]
    at = pe + 24 + opt_size
    for i in range(n):
        o = at + i * 40
        vsize, va, rsize, raw = struct.unpack_from("<IIII", b, o + 8)
        yield va, max(vsize, rsize), raw


def to_off(b: bytes, pe: int, rva: int) -> int:
    for va, size, raw in sections(b, pe):
        if va <= rva < va + size:
            return rva - va + raw
    raise ValueError(f"RVA {rva:#x} 가 어느 섹션에도 없다")


def cstr(b: bytes, off: int) -> str:
    return b[off : b.index(b"\0", off)].decode("ascii", "replace")


def main() -> int:
    b = Path(sys.argv[1]).read_bytes()
    pe = struct.unpack_from("<I", b, 0x3C)[0]
    if b[pe : pe + 4] != b"PE\0\0" or struct.unpack_from("<H", b, pe + 24)[0] != 0x20B:
        print("PE32+ 가 아니다", file=sys.stderr)
        return 1
    opt = pe + 24
    dirs = opt + 112  # PE32+ 데이터 디렉터리
    imp_rva = struct.unpack_from("<I", b, dirs + 1 * 8)[0]
    cfg_rva, cfg_size = struct.unpack_from("<II", b, dirs + 10 * 8)
    names = []
    off = to_off(b, pe, imp_rva)
    while True:
        name_rva = struct.unpack_from("<I", b, off + 12)[0]
        if name_rva == 0:
            break
        names.append(cstr(b, to_off(b, pe, name_rva)))
        off += 20
    flag = 0
    if cfg_rva and cfg_size:
        cfg = to_off(b, pe, cfg_rva)
        # IMAGE_LOAD_CONFIG_DIRECTORY64.DependentLoadFlags(WORD) — 오프셋 0x4E(CSDVersion 다음), 구조 크기가 그보다 커야 있다.
        if struct.unpack_from("<I", b, cfg)[0] >= 0x50:
            flag = struct.unpack_from("<H", b, cfg + 0x4E)[0]
    for n in names:
        print(n)
    vc = sum(n.lower().startswith("vcruntime") for n in names)
    print(f"imports={len(names)} vcruntime={vc} dependent_load_flag={flag:#x}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
