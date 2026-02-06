# Annotator

A Rust CLI tool for reviewing and annotating large repositories file-by-file. Annotations (file + line range + free text) are stored locally and can be passed to AI agents for fixing. Tracks review progress, auto-adjusts annotation positions when code changes via git, and supports session resume.

## Installation

```sh
cargo install --path .
```

## Usage

### TUI Review Mode

```sh
annotator review [path]
```

Opens a full-screen terminal UI for reviewing files. Features:
- Syntax-highlighted file viewer with line numbers
- Gutter markers (`>`) for annotated lines
- Annotation preview in the status bar when cursor is on an annotated line
- Session auto-save and restore (cursor position, scroll, current file)
- Auto-adjusts annotation positions when new commits are detected on startup

### Adjust Annotations

```sh
annotator adjust [path] [--auto-resolve]
```

Headless annotation position adjustment after code changes. Computes `git diff` since the last adjustment and shifts, deletes, or flags conflicts for each annotation. Handles renames, multi-hunk diffs, partial overlaps, and complete rewrites.

### Export Annotations

```sh
annotator export [path] [--format markdown|json]
```

Dumps all annotations to stdout. Markdown (default) groups annotations by file with line references. JSON outputs a structured format suitable for programmatic consumption.

### Review Status

```sh
annotator status [path]
```

Prints a progress summary: total files, unreviewed, annotated, clean, annotation count, and completion percentage.

## TUI Key Bindings

| Key | Action |
|-----|--------|
| Arrows | Navigate cursor |
| Shift+Arrows | Extend selection |
| PgUp / PgDn | Scroll viewport |
| Home / End | Start / end of line |
| Enter | Create annotation for selection or current line |
| Ctrl+E | Edit annotation under cursor |
| Ctrl+D | Delete annotation under cursor |
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Ctrl+M | Mark file as clean (auto-advances to next) |
| Ctrl+N | Jump to next unreviewed file |
| Ctrl+F | Open file list with glob filter |
| Ctrl+T | Open directory tree browser |
| Ctrl+Q | Quit (auto-saves session) |

## Storage

All data is stored in `.annotator/` inside the target repository:

```
.annotator/
  annotations.jsonl    # one annotation per line (append-friendly)
  file_status.jsonl    # file review states
  session.json         # cursor position, last file, last adjust commit
```

Add `.annotator/` to `.gitignore` to keep annotations local, or commit it to share review state with a team.

## Architecture

Library + CLI design: core logic is headless, TUI is one frontend.

```
src/
  cli.rs               # clap subcommand definitions
  main.rs              # CLI dispatch and TUI event loop
  lib.rs               # library re-exports
  core/                # data models, persistence, undo
    annotation.rs      # Annotation, FileStatus, AdjustResult
    store.rs           # JSONL read/append/atomic-rewrite
    session.rs         # session state save/load
    file_list.rs       # git-tracked file enumeration, binary detection
    undo.rs            # undo/redo stack
  git/                 # git integration
    repo.rs            # git2 wrapper
    diff.rs            # FileDiff, Hunk, DiffLine types
    adjust.rs          # annotation position adjustment algorithm
    rename.rs          # rename detection and path migration
  export/              # output formats
    markdown.rs
    json.rs
  tui/                 # terminal UI
    app.rs             # app state machine
    event.rs           # crossterm event polling
    viewer.rs          # file viewer with gutter
    highlight.rs       # syntect to ratatui span conversion
    selection.rs       # shift+arrow text selection
    keymap.rs          # key binding definitions
    render.rs          # layout orchestration
    status_bar.rs      # status line with hotkey hints
    annotation_popup.rs
    file_list_popup.rs
    tree_view.rs
    conflict_popup.rs
```

## Adjustment Algorithm

When `annotator adjust` runs:
1. Reads `last_adjust_commit` from session
2. Computes `git diff <last_commit>..HEAD` with rename detection
3. For each annotation in a changed file:
   - File deleted → annotation removed
   - File renamed → path updated, then hunks processed
   - Hunk before annotation → accumulate line offset
   - Hunk after annotation → stop
   - Hunk overlaps → use line-level diff to detect exact deleted lines
4. All annotated lines deleted → removed
5. Some annotated lines deleted → conflict
6. Only shifted → line numbers updated

## License

MIT
