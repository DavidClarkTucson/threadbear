---
id: autoEverywhere
name: Auto for every thread
title: Auto for every thread: every new Claude thread starts in Auto, whatever it asked for
added: 2026-11-25
on: false
reminder: 7e4e9d97
---

## What it does

Every new Claude thread starts in Auto, whatever mode it was started with: Full access or Ask before edits picked in the new-thread box, a schedule, an email, or another thread's create_thread. Each action is checked, nothing waits on you, and what's refused waits in Permission requests, where Always allow slowly builds up the rules for each folder. Plan first still starts as Plan first, and Codex threads keep the mode they asked for, since Auto there waits on cards like Ask before edits. Threads that already exist keep their mode, and you can still switch any thread by hand.

## Lives in

`startingMode` in `runner/src/runner/new-threads.ts`, called where `create` sets the new thread's `permissionMode`.

## Taking it out

Delete `autoEverywhere.md` and `auto-everywhere.test.ts`, and the `experimentOn` line in `startingMode` (or `startingMode` itself, putting back `params.permissionMode ?? this.cfg.defaults.permissionMode ?? 'full'`).
