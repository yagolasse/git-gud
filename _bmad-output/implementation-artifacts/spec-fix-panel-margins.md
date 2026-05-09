---
title: 'Fix panel margins, background fills, and vertical resizing'
type: refactor
created: '2026-05-05'
status: 'done'
route: 'one-shot'
---

# Fix panel margins, background fills, and vertical resizing

## Intent

**Problem:** The three-panel layout had strange margins (default egui Frame inner_margin), background colors that only covered part of areas (CentralPanel had no fill, leaving transparent gaps between TopBottomPanels), and panels that didn't resize vertically properly (fixed default_heights with conflicting max_height constraints).

**Approach:** Unified all frames with `inner_margin: ZERO, outer_margin: ZERO`. Added a fill to CentralPanel to cover the gap between top/bottom sub-panels. Replaced fixed-height defaults with proportional sizes computed from `available_height` each frame. Removed `max_height` constraints that fought against min_height limits on sibling panels. Applied zero-margin frames to right-side sub-panels for consistency.

## Suggested Review Order

1. [Show main UI method](./src/ui/main_window.rs:340-430) — The complete refactored `show_main_ui` with unified frame pattern, proportional sizing, and CentralPanel fill.
