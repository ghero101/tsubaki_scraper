#!/usr/bin/env python3
"""
Test script to trigger a browser-based download and show bypass feature logs
"""
import requests
import sys

API_URL = "http://127.0.0.1:8080"

# Test with Hive Toons (source_id 20) - uses browser automation
manga_id = "55a9d43b-e4a9-4ef3-bda2-ea053b471073"
chapter_number = "Chapter 1"

print("=" * 70)
print("TESTING ADVANCED CLOUDFLARE BYPASS FEATURES")
print("=" * 70)
print()
print(f"Manga ID: {manga_id}")
print(f"Chapter: {chapter_number}")
print(f"Source: Hive Toons (browser automation with bypass features)")
print()
print("Triggering download... Check server logs for bypass feature messages:")
print("  - [LOCK] Advanced Cloudflare Bypass status")
print("  - [MASK] Fingerprint spoofing injection")
print("  - [WARN] CAPTCHA detection (if present)")
print("  - [REFRESH] Session cookie restoration (if available)")
print("  - [SAVE] Session cookie saving")
print()

response = requests.get(
    f"{API_URL}/download/{manga_id}/{chapter_number}?stream=true",
    stream=True,
    timeout=120
)

if response.status_code == 200:
    total_size = 0
    for chunk in response.iter_content(chunk_size=8192):
        total_size += len(chunk)

    size_mb = total_size / (1024 * 1024)
    print(f"[OK] Download successful! Size: {size_mb:.2f} MB")
    print()
    print("Check the server logs above for bypass feature activity")
else:
    print(f"[FAIL] Download failed: HTTP {response.status_code}")
    print(response.text[:500])

print()
print("=" * 70)
