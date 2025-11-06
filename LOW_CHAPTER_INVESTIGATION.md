# Low Chapter Source Investigation Checklist

## Overview

These 9 sources return manga successfully but only find ~1 chapter per manga (3 total chapters for 3 manga tested). We need to determine if:

1. The sites actually have chapter lists available
2. Chapters are loaded via JavaScript/AJAX
3. Our selectors are wrong
4. Sites have limited free content

## How to Check Each Source

For each source below:

1. **Visit the manga list page** (URL provided)
2. **Click on the FIRST manga** in the list
3. **Look for a chapter list** on the manga detail page
4. **Record your findings** using the template below

### What to Look For

- **Chapter list visible?** Can you see chapters without clicking anything?
- **Chapters in dropdown/tab?** Do you need to click "Chapters" tab?
- **Infinite scroll?** Do chapters load as you scroll?
- **AJAX loaded?** Open DevTools → Network → XHR, reload page, see if chapters load via AJAX
- **JavaScript required?** Disable JavaScript in DevTools, reload - do chapters disappear?

---

## 🔍 Sources to Check

### 1. StoneScape
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://stonescape.xyz/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga (should be "Noa-senpai wa Tomodachi" based on test)
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)
- **Selector hint:** Look for elements like `li.wp-manga-chapter`, `div.eplister`, `div.chapter-list`

---

### 2. Asmotoon
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://asmotoon.com/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

### 3. HiveToons
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://hivetoons.org/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga (should be "Regressing as the Reincarnated Bastard")
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

### 4. KenScans
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://kenscans.com/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

### 5. QIScans
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://qiscans.org/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

### 6. NyxScans
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://nyxscans.com/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga (should be "Operation: True Love")
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

### 7. AsuraScans
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://asuracomic.net/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga (should be "Nano Machine")
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)
- **Note:** AsuraScans is known for Cloudflare protection

---

### 8. MavinTranslations
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://mavintranslations.com/manga/?page=1

**Steps:**
1. Visit manga list
2. Click first manga (should be "The Marquis' Ducal Son")
3. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

### 9. FlameComics
**Status:** 10 manga found, 3 chapters total
**Manga List:** https://flamecomics.com/home

**Steps:**
1. Visit home page
2. Navigate to manga section
3. Click first manga
4. Look for chapter list

**Record:**
- [ ] Can see chapters? (Yes/No)
- [ ] How many chapters visible: ___________
- [ ] Chapters location: □ Below manga info □ In tab □ Hidden □ Other: _____
- [ ] JavaScript required? (Yes/No)
- [ ] AJAX loaded? (Yes/No)

---

## 📋 Summary Template

After checking all sources, fill this out:

### Sources with Visible Chapters (should work with better selectors)
- [ ] StoneScape - _____ chapters
- [ ] Asmotoon - _____ chapters
- [ ] HiveToons - _____ chapters
- [ ] KenScans - _____ chapters
- [ ] QIScans - _____ chapters
- [ ] NyxScans - _____ chapters
- [ ] AsuraScans - _____ chapters
- [ ] MavinTranslations - _____ chapters
- [ ] FlameComics - _____ chapters

### Sources Needing JavaScript/Browser
- [ ] _________________
- [ ] _________________
- [ ] _________________

### Sources with Limited Free Content
- [ ] _________________
- [ ] _________________

### Additional Notes

Copy and paste the HTML structure for any source where you can see chapters. For example:

```html
<!-- Example structure for StoneScape chapters -->
<div class="chapter-list">
  <ul class="chapters">
    <li><a href="/chapter-1">Chapter 1</a></li>
    <li><a href="/chapter-2">Chapter 2</a></li>
  </ul>
</div>
```

This will help me create the correct selectors!

---

## What To Do With Your Findings

Once you've checked all sources, let me know:

1. **Which sources have visible chapters** - I can fix the selectors
2. **Which need JavaScript** - I'll add browser support
3. **Which have limited content** - We can document as expected
4. **Any HTML structures you found** - I'll use them to write better selectors

Thank you! This manual check will help us unlock potentially 5-7 more sources! 🚀
