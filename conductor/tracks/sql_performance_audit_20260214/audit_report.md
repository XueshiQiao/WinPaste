# SQL Performance Audit Report

**Date:** 2026-02-14
**Status:** Audit Complete (Re-verified)

## Summary
The audit identified **2 Critical**, **3 General**, and **Several Negligible** issues.

## Findings Table

| ID | Severity | Location | Description | Frequency | Recommendation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **SQL-01** | **Critical** | `src-tauri/src/clipboard.rs:190` | Fetching `ignore_ghost_clips` setting from DB on every clipboard event. | High (Every Copy) | Cache this setting in memory (e.g., `Arc<AtomicBool>`) and update it only when settings change. |
| **SQL-02** | **Critical** | `src-tauri/src/commands.rs` | `save_settings` executes ~20 separate `INSERT OR REPLACE` queries sequentially. | Medium (On Save) | Use a single transaction or batch the writes into fewer queries (or JSON file). |
| **SQL-03** | **General** | `src-tauri/src/commands.rs` | `get_settings` executes ~20 separate `SELECT` queries to fetch keys one by one. | Medium (On Load) | Replace with JSON file storage. |
| **SQL-05** | **General** | `src-tauri/src/commands.rs:105` | `get_clips` fetches `SELECT *` including potentially large `content` blobs for the list view. | High (UI Refresh) | Select only necessary columns. (DEFERRED) |
