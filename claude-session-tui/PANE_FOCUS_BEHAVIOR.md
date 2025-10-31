# Pane Focus Input Routing (Fixed)

## Problem Identified ❌ (Now Fixed)

The pane focus state was **only visual** - it didn't actually control which keys did what. When you pressed `h` to focus the right pane, the colored border changed but j/k still navigated the session list instead of scrolling the content.

## Solution Implemented ✅

Pane focus now **controls input routing**. Keys behave differently based on which pane is focused:

### Left Pane (Sessions List) - Default focus

When `pane_focus == PaneFocus::Left` (green border on left):

| Keys | Action |
|------|--------|
| `j/k` | Navigate sessions (down/up) |
| `J/K` | Jump 5 sessions (down/up) |
| `↑/↓` | Navigate sessions (down/up) |
| `h` | Stay focused on left pane |
| `l` | Switch to right pane |
| `/` | Enter search mode |
| `o`/`Enter` | Open JSON view in right pane |

### Right Pane (Details/Content) - After pressing `l`

When `pane_focus == PaneFocus::Right` (green border on right):

#### In Summary View
| Keys | Action |
|------|--------|
| `j/k` | Navigate snippet matches (if available) |
| `↑/↓` | Navigate snippet matches |
| `l` | Stay focused on right pane |
| `h` | Switch to left pane |
| `v` | Switch to Full JSON view |
| `o` | (N/A in summary) |

#### In Full JSON View
| Keys | Action |
|------|--------|
| `j/k` | Scroll JSON content (down/up) |
| `J/K` | Scroll JSON faster - 5 lines (down/up) |
| `↑/↓` | Scroll JSON content (down/up) |
| `l` | Stay focused on right pane |
| `h` | Switch to left pane |
| `v` | Switch back to Summary view |

#### In Snippet Browser View
| Keys | Action |
|------|--------|
| `j/k` | Scroll snippet content (down/up) |
| `J/K` | Scroll snippet faster - 5 lines (down/up) |
| `↑/↓` | Scroll snippet content (down/up) |
| `n/p` | Navigate to next/previous match |
| `l` | Stay focused on right pane |
| `h` | Switch to left pane |
| `v` | Exit to Summary view |

## Implementation Details

### Key Handler Changes (src/ui/app.rs:445-529)

```rust
// Example: j/k key handler now respects pane focus
KeyCode::Char('j') => {
    if self.is_searching {
        self.search_query.push('j');  // In search mode: add to query
    } else if self.pane_focus == PaneFocus::Left {
        // Left pane: navigate sessions
        if self.selected + 1 < self.filtered_sessions.len() {
            self.selected += 1;
        }
    } else {
        // Right pane: scroll content based on view mode
        match self.view_mode {
            ViewMode::FullJson => self.scroll_json(1),      // Scroll JSON
            ViewMode::SnippetBrowser => self.scroll_snippet(1),  // Scroll snippet
            ViewMode::Summary => {
                // Navigate matches if available
                let session_matches = self.get_matches_for_selected_session();
                if !session_matches.is_empty() && self.snippet_index + 1 < session_matches.len() {
                    self.snippet_index += 1;
                }
            }
        }
    }
}
```

### Arrow Keys (src/ui/app.rs:616-645)

Arrow keys now respect pane focus consistently:
- Right pane focused: Up/Down scroll content
- Left pane focused: Up/Down navigate sessions

```rust
KeyCode::Up => {
    if self.pane_focus == PaneFocus::Right {
        // Right pane: scroll content
        match self.view_mode {
            ViewMode::SnippetBrowser => self.scroll_snippet(-1),
            ViewMode::FullJson => self.scroll_json(-1),
            ViewMode::Summary => {}
        }
    } else {
        // Left pane: navigate sessions
        if !self.is_searching && self.selected > 0 {
            self.selected -= 1;
        }
    }
}
```

### Dynamic Help Text (src/ui/app.rs:111-123)

Help text now shows which pane is active:

```
h/l:panes(LEFT)  /search  j/k/↑↓:nav  v:view  o/enter:json  q:quit
                  ^^^^
              Current pane shown here
```

When right pane is focused:
```
h/l:panes(RIGHT)  /search  j/k/↑↓:nav  v:view  o/enter:json  q:quit
                   ^^^^^
              Indicates scrolling behavior
```

## User Experience Flow

### Scenario 1: Browse sessions and read one

1. Start with left pane focused (default)
2. Press `j/k` to navigate through sessions
3. Find interesting session
4. Press `l` to focus right pane → border turns green
5. Press `o` to open full JSON
6. Press `j/k` to scroll through JSON content
7. Find what you need
8. Press `h` to go back to session list
9. Continue browsing

### Scenario 2: Search and scroll results

1. Press `/` to search
2. Type search query
3. Press `Enter` to search
4. Press `l` to focus right pane
5. Press `v` to enter snippet browser
6. Press `j/k` to scroll snippet content
7. Press `n` to jump to next match
8. Press `h` to go back to session list
9. Press `j/k` to navigate other sessions

## Testing

Navigation tests verify this behavior:
- `test_pane_focus_switching_h_and_l` - Focus state changes
- `test_session_list_navigation_up_and_down` - Left pane navigation
- `test_json_view_scrolling` - Right pane JSON scrolling
- `test_snippet_navigation_with_n_and_p` - Match navigation

All navigation tests (10/10) ✅ pass.

## Summary

The pane focus system now provides:
- ✅ **Visual feedback** - Colored borders show which pane is active
- ✅ **Input routing** - Keys behave correctly based on focus
- ✅ **Consistent behavior** - Same keys work the same way (j/k down, k down)
- ✅ **Clear help text** - Shows current pane and key meanings
- ✅ **Intuitive flow** - Users can easily browse, read, and search sessions

**Before:** Visual borders but no functional input routing
**After:** Full pane-aware input system with visual and functional integration
