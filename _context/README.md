# _context/

Persistent knowledge base for the Spout project. All agents read this before
significant work and update it when they learn something worth keeping.

## Structure

```
_context/
  README.md              ← this file (index + process)
  wasm-debugging.md      ← WASM/WebGPU gotchas and debugging workflow
  plans/
    active/              ← one file per in-progress initiative
    archive/             ← completed or superseded plans/docs
```

## What lives here

| File/dir | Purpose |
|----------|---------|
| `wasm-debugging.md` | Concrete WASM gotchas with fixes; WGSL pitfalls; browser debugging workflow |
| `plans/active/` | In-progress plans — decisions made, scope, status, open items |
| `plans/archive/` | Completed plans and old assessments (preserved for history) |

## Agent process

**At the start of a session:**
1. Read `AGENTS.md` (build commands, architecture, key constraints).
2. Skim `_context/README.md` (this file) to know what context exists.
3. Read any `_context/plans/active/` files relevant to the work at hand.
4. Read `_context/wasm-debugging.md` before touching WASM, shaders, or the
   browser rendering path.

**During work:**
- If a plan file exists for your task, keep its status current as you go.

**At the end of a session:**
1. Update any plan files touched — mark completed items, note new open issues.
2. If you hit a non-obvious bug or platform-specific gotcha, add it to
   `wasm-debugging.md` (or a new file if it doesn't fit there).
3. When a plan is fully complete, move it from `plans/active/` to
   `plans/archive/`.
4. If AGENTS.md has stale information (e.g. wrong dep versions), fix it.

## What does NOT live here

- Anything already in the code or git history — don't duplicate.
- In-session scratch notes or task lists — use tasks/todos for that.
- Personal preferences about Claude's behavior — use CLAUDE.md for that.
