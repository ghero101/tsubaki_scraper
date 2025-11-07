#!/usr/bin/env python3
import re

# Read the HTML file
with open('asuracomics_series.html', 'r', encoding='utf-8') as f:
    html = f.read()

# Test the regex pattern from the Rust code
pattern = r'href="([^"]*chapter/(\d+)[^"]*)"'
matches = re.findall(pattern, html)

print(f"Total chapter matches: {len(matches)}")
print("\nFirst 10 matches:")
for i, (full_url, chapter_num) in enumerate(matches[:10]):
    print(f"  {i+1}. Chapter {chapter_num}: {full_url}")

# Deduplicate by chapter number
unique_chapters = {}
for full_url, chapter_num in matches:
    num = int(chapter_num)
    unique_chapters[num] = full_url

print(f"\nUnique chapters: {len(unique_chapters)}")
print(f"Chapter range: {min(unique_chapters.keys())} - {max(unique_chapters.keys())}")
