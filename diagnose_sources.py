#!/usr/bin/env python3
"""
Diagnostic script to analyze low-chapter sources
Fetches manga pages and analyzes HTML structure
"""

import requests
import json
from html.parser import HTMLParser

sources = {
    "StoneScape": "https://stonescape.xyz",
    "FlameComics": "https://flamecomics.com",
    "Asmotoon": "https://asmotoon.com",
    "HiveToons": "https://hivetoons.org",
    "KenScans": "https://kencomics.com",
    "QIScans": "https://qiscans.org",
    "NyxScans": "https://nyxscans.com",
    "AsuraScans": "https://asuracomic.net",
    "MavinTranslations": "https://mavintranslations.com",
}

headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
}

print("=" * 80)
print("MANGA SOURCE DIAGNOSIS")
print("=" * 80)

for name, url in sources.items():
    print(f"\n{'='*80}")
    print(f"Testing: {name} ({url})")
    print("=" * 80)

    try:
        # Try homepage first
        resp = requests.get(url, headers=headers, timeout=10)
        html = resp.text

        print(f"[OK] Status: {resp.status_code}")
        print(f"[OK] Content length: {len(html)} bytes")

        # Detect technology
        if "_next" in html or "nextjs" in html.lower() or "__NEXT_DATA__" in html:
            print("[TECH] Next.js/React (JavaScript rendered)")
        elif "wp-content" in html or "wordpress" in html.lower():
            print("[TECH] WordPress")
        elif "madara" in html.lower() or "ct-icon" in html:
            print("[TECH] Madara theme (WordPress)")
        else:
            print("[TECH] Unknown/Custom")

        # Check for common chapter selectors
        selectors_found = []
        if "wp-manga-chapter" in html:
            selectors_found.append("wp-manga-chapter")
        if "chapter-item" in html:
            selectors_found.append("chapter-item")
        if "list-chapter" in html:
            selectors_found.append("list-chapter")
        if "eplister" in html:
            selectors_found.append("eplister")
        if '"chapters":' in html or '"chapter":' in html:
            selectors_found.append("JSON data (chapters)")

        if selectors_found:
            print(f"[FOUND] Selectors: {', '.join(selectors_found)}")
        else:
            print("[WARN] No common chapter selectors found")

        # Check if JavaScript heavy
        script_count = html.count("<script")
        if script_count > 20:
            print(f"[WARN] JavaScript-heavy ({script_count} script tags) - may need browser")

    except requests.exceptions.Timeout:
        print("[ERROR] Timeout after 10 seconds")
    except Exception as e:
        print(f"[ERROR] {e}")

print("\n" + "="*80)
print("DIAGNOSIS COMPLETE")
print("="*80)
