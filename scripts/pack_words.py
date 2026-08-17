#!/usr/bin/env python3
"""Packs top frequency English words into assets/words.bin for QuickyNotes."""

import struct
import urllib.request

URL = "https://norvig.com/ngrams/count_1w.txt"
FALLBACK_URL = "https://raw.githubusercontent.com/david47k/top-english-wordlists/master/top_english_words_lower_50000.txt"


def main():
    print("Fetching top English wordlist (333k Norvig Google Web dataset)...")
    req = urllib.request.Request(URL, headers={"User-Agent": "Mozilla/5.0"})
    try:
        with urllib.request.urlopen(req, timeout=15) as r:
            lines = r.read().decode("utf-8", errors="ignore").splitlines()
            raw_words = [line.split("\t")[0] for line in lines if line.strip()]
    except Exception as e:
        print(f"Warning: Primary URL failed ({e}), using fallback...")
        req = urllib.request.Request(FALLBACK_URL, headers={"User-Agent": "Mozilla/5.0"})
        with urllib.request.urlopen(req, timeout=15) as r:
            raw_words = r.read().decode("utf-8", errors="ignore").splitlines()

    cleaned = []
    seen = set()
    for w in raw_words:
        w = w.strip().lower()
        if not w or len(w) < 2 or len(w) > 35:
            continue
        if not all(c.isalnum() or c in "-_'" for c in w):
            continue
        if w not in seen:
            seen.add(w)
            cleaned.append(w)

    total = len(cleaned)
    print(f"Cleaned {total} words.")

    buf = bytearray(b"QNW1")
    for i, w in enumerate(cleaned):
        weight = max(1, int(65535 * (1.0 - (i / total))))
        w_bytes = w.encode("utf-8")
        buf.append(len(w_bytes))
        buf.extend(w_bytes)
        buf.extend(struct.pack("<H", weight))

    out_path = "assets/words.bin"
    with open(out_path, "wb") as f:
        f.write(buf)

    print(f"Wrote {len(buf)} bytes ({len(buf)/1024:.1f} KB) to {out_path}")


if __name__ == "__main__":
    main()
