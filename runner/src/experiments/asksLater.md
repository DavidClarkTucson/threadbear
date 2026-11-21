---
id: asksLater
name: Ask me later
title: Ask me later: Auto threads never wait on you; what they're refused waits in Permission requests
added: 2026-11-21
on: false
reminder: 00000000
---

## What it does

An Auto thread no longer stops to wait for you. When its safety check and the card reviewer won't allow something, Threadbear refuses it straight away, the agent carries on without it, and its final reply says what it couldn't do. The question goes to one thread, Permission requests, as a card you answer whenever you like. **Always allow** saves an exact rule for that folder (you pick how broad: just `npm test`, or any `npm test …`; one file, or its folder), so it isn't stopped again. **Allow once** tells the thread to try that one call again, and lets it through. **Keep blocking** changes nothing. Actions that hand out lasting access (more access for a thread, schedules, moving threads) and edits to files Threadbear runs as code can only ever be allowed once, and only from a Mac. Claude threads only; Ask before edits and Full access don't change.

## Lives in

`runner/src/runner/asks-later.ts` (rules, the store `asks-later.json` in the data folder, the Permission requests thread), its two calls in `reviewThenCard` and the `savable` flag in `requestApproval` in `runner/src/runner/approvals.ts`, `repostLaterAsks` in `Runner.start` and a question card's `macOnly` in `runner/src/runner.ts`.

## Taking it out

Delete `asks-later.ts` and `asks-later.test.ts`, the `later`/`earlier`/`blockForLater` lines in `reviewThenCard` and `savable` in `requestApproval`, `repostLaterAsks` in `start`, and `asksLaterPart` from the runner's parts. The question card's `macOnly` can stay.
