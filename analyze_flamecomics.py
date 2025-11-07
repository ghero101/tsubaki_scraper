#!/usr/bin/env python3
"""Analyze FlameComics Next.js data"""

import json
import re

# Read HTML
with open('flamecomics_series.html', 'r', encoding='utf-8') as f:
    html = f.read()

# Find __NEXT_DATA__ JSON
match = re.search(r'<script id="__NEXT_DATA__" type="application/json">(.+?)</script>', html, re.DOTALL)
if not match:
    print("ERROR: Could not find __NEXT_DATA__")
    exit(1)

json_str = match.group(1)
data = json.loads(json_str)

print("="*80)
print("FLAMECOMICS API STRUCTURE")
print("="*80)

print("\nTop-level keys:", list(data.keys()))
print("Page:", data.get('page'))

props = data.get('props', {}).get('pageProps', {})
print("\nPageProps keys:", list(props.keys()))

# Analyze series data
series = props.get('series', {})
if series:
    print("\n" + "="*80)
    print("SERIES DATA")
    print("="*80)
    print(f"Title: {series.get('title')}")
    print(f"ID: {series.get('id')}")
    print(f"Description: {series.get('description', '')[:100]}...")
    print(f"\nSeries keys: {list(series.keys())}")

    # Check for chapters
    chapters = series.get('chapters', [])
    if chapters:
        print(f"\n[SUCCESS] Found {len(chapters)} chapters!")
        print("\nFirst chapter structure:")
        print(json.dumps(chapters[0], indent=2))

        print("\nLast chapter structure:")
        print(json.dumps(chapters[-1], indent=2))
    else:
        print("\n[WARN] No chapters in series object")
        print("Let me check alternative locations...")

        # Check other possible locations
        for key in props.keys():
            if 'chapter' in key.lower():
                print(f"Found key with 'chapter': {key} = {props[key][:100] if isinstance(props[key], str) else type(props[key])}")

# Check for API endpoints in the data
api_hints = []
for key, value in props.items():
    if isinstance(value, str) and ('api' in value.lower() or 'endpoint' in value.lower()):
        api_hints.append((key, value))

if api_hints:
    print("\n" + "="*80)
    print("POSSIBLE API ENDPOINTS")
    print("="*80)
    for key, value in api_hints:
        print(f"{key}: {value}")

print("\n" + "="*80)
print("RAW pageProps keys for manual inspection:")
for key in props.keys():
    value = props[key]
    value_type = type(value).__name__
    if isinstance(value, (list, dict)):
        length = len(value)
        print(f"  {key}: {value_type} (len={length})")
    else:
        print(f"  {key}: {value_type}")
