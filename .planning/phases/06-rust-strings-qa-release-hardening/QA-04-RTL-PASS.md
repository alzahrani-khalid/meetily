# Manual QA Pass -- Phase 6

**Date:** ___
**Tester:** ___
**App version:** ___
**Platform:** macOS ___

---

## QA-04: RTL Regression Pass

Pass bar (D-08): No text overflow, clipping, or visual asymmetry on any of the 7 screens below. Arabic averages ~1.2x English width -- all labels must fit without truncation.

**Pre-conditions:**
- App running with `ui_locale='ar'`
- Tajawal font loaded (verify in DevTools: `document.fonts.check('16px Tajawal')`)
- `<html dir="rtl" lang="ar">` present in DOM

### Screen 1: Sidebar

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 1.1 | Sidebar opens from the right edge | [ ] | [ ] | |
| 1.2 | Collapse animation slides toward right edge in RTL | [ ] | [ ] | UI-06 critical |
| 1.3 | Collapse animation slides toward left edge in LTR (switch locale, verify) | [ ] | [ ] | |
| 1.4 | Active meeting item highlight on correct (right) side | [ ] | [ ] | |
| 1.5 | Meeting list items: text right-aligned, no overflow | [ ] | [ ] | |
| 1.6 | Sidebar icons positioned correctly (start edge) | [ ] | [ ] | |
| 1.7 | No horizontal scrollbar visible | [ ] | [ ] | |

### Screen 2: Settings Modal

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 2.1 | All labels right-aligned in RTL | [ ] | [ ] | |
| 2.2 | Radio groups on correct side (start edge = right in RTL) | [ ] | [ ] | |
| 2.3 | Switches positioned on correct side | [ ] | [ ] | |
| 2.4 | No clipping on Arabic labels (1.2x width factor) | [ ] | [ ] | |
| 2.5 | Language switcher section displays correctly | [ ] | [ ] | |
| 2.6 | Model settings dropdowns open in correct direction | [ ] | [ ] | |
| 2.7 | Close button (X) positioned on correct side (start = right in RTL) | [ ] | [ ] | |

### Screen 3: Transcript Panel

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 3.1 | Timestamps aligned to start edge (right in RTL) | [ ] | [ ] | |
| 3.2 | Transcript text flows RTL with `writingDirection: rtl` | [ ] | [ ] | |
| 3.3 | No horizontal scroll on long Arabic text | [ ] | [ ] | |
| 3.4 | Speaker labels right-aligned | [ ] | [ ] | |
| 3.5 | Copy/export buttons positioned correctly | [ ] | [ ] | |

### Screen 4: Summary Panel

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 4.1 | BlockNote editor content renders RTL | [ ] | [ ] | SUMM-04 |
| 4.2 | Template selector labels display in Arabic | [ ] | [ ] | |
| 4.3 | No overflow on long Arabic headings | [ ] | [ ] | |
| 4.4 | Summary section headers right-aligned | [ ] | [ ] | |
| 4.5 | Action buttons (regenerate, copy) positioned correctly | [ ] | [ ] | |

### Screen 5: Onboarding Flow

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 5.1 | All onboarding steps right-aligned in RTL | [ ] | [ ] | |
| 5.2 | Progress indicator flows correct direction (RTL) | [ ] | [ ] | |
| 5.3 | CTA buttons full-width, centered text | [ ] | [ ] | |
| 5.4 | Whisper model download progress displays correctly | [ ] | [ ] | |
| 5.5 | Parakeet option NOT shown for Arabic locale | [ ] | [ ] | TRANS-02 |

### Screen 6: Tray Menu

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 6.1 | All 13 tray items display in Arabic | [ ] | [ ] | UI-07 |
| 6.2 | No truncation on primary items (Start/Stop/Pause/Resume) | [ ] | [ ] | |
| 6.3 | Emoji prefixes visible on all items | [ ] | [ ] | D-03 |
| 6.4 | Tray rebuilds to Arabic after locale switch (no app restart) | [ ] | [ ] | |
| 6.5 | Tray shows Arabic from first paint when `ui_locale='ar'` at startup | [ ] | [ ] | |

### Screen 7: Meeting Details

| # | Check | Pass | Fail | Notes |
|---|-------|------|------|-------|
| 7.1 | Meeting name label in Arabic | [ ] | [ ] | |
| 7.2 | Date and duration labels in Arabic | [ ] | [ ] | |
| 7.3 | Action buttons (delete, export) use destructive color correctly | [ ] | [ ] | |
| 7.4 | No text overflow on Arabic meeting names | [ ] | [ ] | |
| 7.5 | Import audio dialog renders correctly in RTL | [ ] | [ ] | |

### QA-04 Summary

| Screen | Total Checks | Pass | Fail |
|--------|-------------|------|------|
| Sidebar | 7 | ___ | ___ |
| Settings Modal | 7 | ___ | ___ |
| Transcript Panel | 5 | ___ | ___ |
| Summary Panel | 5 | ___ | ___ |
| Onboarding Flow | 5 | ___ | ___ |
| Tray Menu | 5 | ___ | ___ |
| Meeting Details | 5 | ___ | ___ |
| **Total** | **39** | ___ | ___ |

**QA-04 Verdict:** [ ] PASS / [ ] FAIL

---

## QA-05: Arabic Transcription Quality Spot-Check

Pass bar (D-09): Recognizable Arabic output, correct sentence structure, no English hallucination. Target ~85-88% accuracy for MSA with Whisper large-v3.

**Pre-conditions:**
- Whisper `large-v3` model downloaded and loaded
- `ui_locale='ar'` and `transcription_language='ar'` set
- MSA Arabic audio sample ready (30-60 seconds)

### Test Matrix

| # | Audio Sample | Duration | Expected Content | Accuracy Estimate | English Hallucination? | Notes |
|---|-------------|----------|-----------------|-------------------|----------------------|-------|
| 5.1 | Sample 1: ___ | ___s | ___ | ___% | [ ] Yes / [ ] No | |
| 5.2 | Sample 2: ___ | ___s | ___ | ___% | [ ] Yes / [ ] No | |
| 5.3 | Sample 3: ___ | ___s | ___ | ___% | [ ] Yes / [ ] No | |

### Observations

- Sentence structure quality: ___
- Diacritics handling: ___
- Punctuation accuracy: ___
- Overall intelligibility: ___

**QA-05 Verdict:** [ ] PASS / [ ] FAIL

---

## QA-06: Arabic Summary Quality Spot-Check

Pass bar (D-10): Output is coherent Arabic, uses correct punctuation marks (`،` `؛` `؟`), no English leakage in body. Both Claude AND Ollama providers must be tested.

**Pre-conditions:**
- Arabic template selected
- Arabic transcript available (from QA-05 or pre-existing)
- Arabic prompt loaded via `prompts::get_prompt(id, "ar")`

### Provider: Claude

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 6.1 | Output is fully Arabic | [ ] Pass / [ ] Fail | |
| 6.2 | Arabic punctuation present (`،` `؛` `؟`) | [ ] Pass / [ ] Fail | |
| 6.3 | No English leakage in body text | [ ] Pass / [ ] Fail | |
| 6.4 | RTL formatting correct when rendered | [ ] Pass / [ ] Fail | |
| 6.5 | Template structure followed correctly | [ ] Pass / [ ] Fail | |

### Provider: Ollama

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 6.6 | Output is fully Arabic | [ ] Pass / [ ] Fail | |
| 6.7 | Arabic punctuation present (`،` `؛` `؟`) | [ ] Pass / [ ] Fail | |
| 6.8 | No English leakage in body text | [ ] Pass / [ ] Fail | |
| 6.9 | RTL formatting correct when rendered | [ ] Pass / [ ] Fail | |
| 6.10 | Template structure followed correctly | [ ] Pass / [ ] Fail | |

### Observations

- Claude output quality: ___
- Ollama output quality: ___
- Punctuation consistency: ___
- Template adherence: ___

**QA-06 Verdict:** [ ] PASS / [ ] FAIL

---

## Overall Phase 6 QA Summary

| Requirement | Verdict | Blocker? |
|-------------|---------|----------|
| QA-04: RTL Regression Pass | [ ] PASS / [ ] FAIL | Yes |
| QA-05: Arabic Transcription | [ ] PASS / [ ] FAIL | Yes |
| QA-06: Arabic Summary Quality | [ ] PASS / [ ] FAIL | Yes |

**Phase 6 Manual QA Verdict:** [ ] PASS / [ ] FAIL

**Sign-off:** ___
**Date:** ___
