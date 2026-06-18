# Observatory Map

## Project Identity

Project name:

```text
Codex JSONL Observatory
```

Purpose:

```text
A local-first Rust + Svelte web viewer for Codex CLI JSONL session logs.
```

This project is not only a viewer.
It is an observatory for AI-assisted work records.

---

## Reference Project

The original Kotlin/Swing implementation is maintained separately.

Original Kotlin/Swing version:

```text
https://github.com/RGJ-sw1123r/codex-chat-viewer
```

Codex JSONL Observatory rebuilds the same idea as a Rust + Svelte local web observatory.

---

## Metaphor Map

Use these meanings consistently:

```text
Signal Records      = raw Codex CLI JSONL files
Signals             = raw JSONL events
Observations        = parsed/rendered entries
Observatory Core    = Rust backend
Control Room        = Svelte frontend
Observation Report  = Markdown export
Sample Signals      = sanitized sample JSONL files
Field Kit           = release package
```

The metaphor exists to clarify structure.
Do not use metaphor when it hides purpose.

---

## Repository Shape

Expected top-level structure:

```text
backend/       Rust Observatory Core
frontend/      Svelte Control Room
docs/          planning, migration notes, and boundary documents
sample-data/   sanitized sample JSONL files only
release/       local release packaging workspace
```

Do not rename top-level areas without explicit approval.

---

## Backend Responsibility

The Rust backend is the Observatory Core.

It should handle:

* reading Codex JSONL files
* parsing raw signals
* creating observations
* preserving parser behavior from the Kotlin/Swing reference where appropriate
* exposing local API endpoints
* generating Observation Reports
* serving built frontend assets in release mode

Prefer clear technical names:

```text
backend/src/parser/
backend/src/domain/
backend/src/api/
backend/src/export/
```

Avoid decorative names that hide purpose.

---

## Frontend Responsibility

The Svelte frontend is the Control Room.

It should handle:

* file/path input flow
* parsed observation display
* search controls
* filter controls
* view mode controls
* collapse/expand interaction
* export action UI
* user-visible status and error display

Good component names:

```text
ControlRoom.svelte
ObservationList.svelte
ObservationEntry.svelte
SignalPanel.svelte
FilterConsole.svelte
SearchConsole.svelte
```

Avoid vague names:

```text
NebulaPanel.svelte
StarMagic.svelte
BlackholeView.svelte
```

The metaphor must clarify boundaries, not decorate confusion.