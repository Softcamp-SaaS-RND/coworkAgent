#!/usr/bin/env python
"""앱 아이콘 생성 — 실행 파일 아이콘(.ico)과 트레이 아이콘(32×32 RGBA) 을 같은 그림(시안 B2: 말풍선 둘 + ai)에서 만든다.

결과는 `crates/app/assets/` 에 커밋한다 — 빌드는 이 스크립트를 부르지 않는다. 그림을 바꿀 때만 다시 돌린다.
Pillow 와 Windows 글꼴 Segoe UI Bold(`segoeuib.ttf`) 가 필요하다(개발 PC 전용).

    python tools/gen_icon.py
"""

import os
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

OUT = Path(__file__).resolve().parent.parent / "crates" / "app" / "assets"
FONT = Path(os.environ.get("WINDIR", "C:/Windows")) / "Fonts" / "segoeuib.ttf"  # Windows 기본 글꼴 폴더
NAVY, RED = "#12224A", "#D93B2B"  # 기본 바탕 · 트레이 오류 모드 바탕
BACK, FRONT = "#5B8CFF", "#FFFFFF"
SS = 4  # 256 좌표계를 1024 로 그려 줄이며 앤티에일리어싱


def draw(ground: str) -> Image.Image:
    """256×256 좌표계 그림을 1024 로 그린다(시안 SVG 좌표 그대로)."""
    im = Image.new("RGBA", (256 * SS, 256 * SS), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)

    def s(*v: float) -> list[float]:
        return [x * SS for x in v]

    d.rounded_rectangle(s(0, 0, 256, 256), radius=56 * SS, fill=ground)
    d.rounded_rectangle(s(28, 36, 148, 116), radius=28 * SS, fill=BACK)
    d.polygon(s(54, 108, 46, 144, 90, 112), fill=BACK)
    d.rounded_rectangle(s(59, 91, 237, 217), radius=41 * SS, fill=ground)
    d.rounded_rectangle(s(64, 96, 232, 212), radius=36 * SS, fill=FRONT)
    d.polygon(s(200, 204, 210, 236, 166, 208), fill=FRONT)
    d.text((148 * SS, 152 * SS), "ai", font=ImageFont.truetype(str(FONT), 96 * SS), fill=NAVY, anchor="mm")
    return im


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    big = draw(NAVY).resize((256, 256), Image.Resampling.LANCZOS)
    big.save(OUT / "coworkAgent.ico", sizes=[(n, n) for n in (16, 20, 24, 32, 40, 48, 64, 128, 256)])
    for name, ground in (("tray.rgba", NAVY), ("tray-error.rgba", RED)):
        (OUT / name).write_bytes(draw(ground).resize((32, 32), Image.Resampling.LANCZOS).tobytes())
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
