"""Assemble verified PBR renders into a labeled review sheet (Python + Pillow)."""

import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont, __version__ as pillow_version


def digest(path):
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--review", type=Path, required=True)
    args = parser.parse_args()
    report_path = args.review / "review.json"
    report = json.loads(report_path.read_text())
    output = args.review / "comparison.png"
    metadata = args.review / "comparison.json"
    if output.exists() or metadata.exists():
        raise ValueError("Comparison files already exist; use a new review run")
    cases = (
        ("detail-min", "01  MINIMUM DETAIL", "detail 0 / grain scale 64"),
        ("default", "02  LEATHER CANDIDATE", "detail 0.35 / grain scale 64"),
        ("detail-max", "03  MAXIMUM DETAIL", "detail 1 / grain scale 64"),
        ("coarse-grain", "04  COARSER GRAIN", "detail 0.35 / grain scale 32"),
    )
    cell, margin, gutter, top = 480, 24, 12, 112
    sheet = Image.new("RGB", (2 * margin + 4 * cell + 3 * gutter, 714), "#10151c")
    draw = ImageDraw.Draw(sheet)
    title = ImageFont.load_default(size=30)
    heading = ImageFont.load_default(size=21)
    body = ImageFont.load_default(size=17)
    draw.text((margin, 22), "LEATHER / MATERIAL REVIEW", font=title, fill="#f2f4f8")
    draw.text((margin, 65), "Actual Mixture PNGs. Identical camera, lighting and BRDF. Human acceptance pending.", font=body, fill="#bcc8d6")
    for index, (case, name, detail) in enumerate(cases):
        path = args.review / f"{case}.png"
        if digest(path) != report["results"][case]["sha256"]:
            raise ValueError(f"Render does not match review evidence: {case}")
        with Image.open(path) as rendered:
            if list(rendered.size) != report["size"]:
                raise ValueError(f"Render dimensions do not match: {case}")
            thumbnail = rendered.convert("RGB").resize((cell, cell), Image.Resampling.LANCZOS)
        x = margin + index * (cell + gutter)
        sheet.paste(thumbnail, (x, top))
        draw.text((x, top + cell + 14), name, font=heading, fill="#f2f4f8")
        draw.text((x, top + cell + 44), detail, font=body, fill="#bcc8d6")
    draw.text((margin, 679), "Actual height-derived normal applied once. No extra bump, displacement, coat, sheen or surface noise.", font=body, fill="#96a5b8")
    sheet.save(output)
    metadata.write_text(json.dumps({
        "schemaVersion": 1,
        "reviewSha256": digest(report_path),
        "scriptSha256": digest(Path(__file__)),
        "pillowVersion": pillow_version,
        "operation": "Labeled contact sheet; Lanczos downsampling only, no relighting or retouching",
        "outputSha256": digest(output),
        "humanAcceptanceClaimed": False,
    }, indent=2) + "\n")


if __name__ == "__main__":
    main()
