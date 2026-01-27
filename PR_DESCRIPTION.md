# Add Real-Time Search Functionality

## Overview

This PR implements comprehensive real-time search functionality that allows users to quickly filter through chat lists and messages within chats. The search is performed in real-time as the user types, with fuzzy matching for flexible results.

---

## Feature Description

### Key Features

#### 1. **Chat List Search**
- Real-time filtering of chat names as you type
- Fuzzy matching using `nucleo_matcher` for flexible search
- Inline search bar displayed at the top of the chat list
- Navigation through filtered results using arrow keys or Tab
- Exit search mode with Enter or Esc

#### 2. **Message Search**
- Real-time filtering of messages within an open chat
- Searches message content using fuzzy matching
- Inline search bar displayed at the top of the chat window
- Navigation through filtered messages using arrow keys or Tab
- Selection preserved by message ID (not index) to prevent jumping to wrong messages
- Exit search mode with Enter or Esc

#### 3. **Smart Keybindings**
- **Alt+R**: 
  - When nothing is selected: Activates chat list search
  - When ChatList is focused: Activates chat list search
  - When Chat is focused: Activates message search
  - When Prompt is focused: Activates message search
  - When in message search mode: Switches to chat list search
- **Alt+C**: Restores default sorting/filtering

---

## Technical Implementation

### Architecture

**Components Modified:**
- `ChatListWindow`: Added search mode state and filtering logic
- `ChatWindow`: Added search mode state and filtering logic
- `CoreWindow`: Enhanced action routing for search functionality
- `PromptWindow`: Added Alt+R handling for quick search access

**New Actions Added:**
- `ChatWindowSearch`: Activates message search mode
- `ChatWindowSortWithString(String)`: Filters messages by search string
- `ChatWindowRestoreSort`: Restores default message ordering
- `ChatListSearch`: Activates chat list search mode (already existed, enhanced)
- `ChatListSortWithString(String)`: Filters chats by search string (already existed, enhanced)
- `ChatListRestoreSort`: Restores default chat ordering (already existed, enhanced)

### Search Algorithm

- Uses `nucleo_matcher` library for fuzzy string matching
- Configures matcher with `prefer_prefix = true` for better UX
- Filters in real-time as user types
- Preserves selection by ID (not index) to maintain correct message position

### State Management

**Search Mode States:**
- `search_mode: bool`: Tracks if component is in search mode
- `search_input: String`: Stores the current search query
- `sort_string: Option<String>`: Used for filtering logic

**Selection Preservation:**
- When filtering, selection is tracked by message/chat ID, not list index
- When exiting search mode, correct item is selected in full list by ID
- Prevents jumping to wrong position when switching between filtered and full lists

### Key Event Handling

- Arrow keys and Tab navigate through filtered results
- Character input updates search query in real-time
- Backspace removes characters and updates filter
- Enter/Esc exits search mode
- Alt+R provides quick access to search from any context

---

## User Experience Improvements

1. **No Double Processing**: Fixed issue where arrow keys moved 2 items instead of 1 in search mode
2. **Smooth Navigation**: Arrow keys work seamlessly in both search and normal modes
3. **Context-Aware Search**: Alt+R intelligently activates appropriate search based on current focus
4. **Quick Switching**: Can switch between chat list and message search with Alt+R
5. **Visual Feedback**: Search bars clearly indicate active search mode

---

## Bug Fixes During Implementation

1. **Double Letter Input**: Fixed keys being processed multiple times by ensuring single event handling
2. **Double Navigation**: Fixed arrow keys moving 2 items by preventing duplicate action processing
3. **Selection Jumping**: Fixed selection jumping to wrong position by tracking by ID instead of index
4. **List Updates**: Prevented filtered lists from updating during search to maintain user's position

---

## Files Changed

### Core Changes
- `src/components/prompt_window.rs`: Added Alt+R handling for quick search access
- `src/components/chat_list_window.rs`: Added search mode, filtering, and inline search bar
- `src/components/chat_window.rs`: Added search mode, filtering, inline search bar, and ID-based selection
- `src/components/core_window.rs`: Enhanced action routing and search mode coordination
- `src/action.rs`: Added new search-related actions

### Configuration
- `config/keymap.toml`: Added Alt+R to core_window keymap for global search access

### Documentation
- `CHANGELOG.md`: Documented new search feature

---

## Testing

- ✅ Real-time filtering works correctly for both chat lists and messages
- ✅ Navigation through filtered results works smoothly (1 item at a time)
- ✅ Selection preservation works correctly (no jumping to wrong items)
- ✅ Alt+R keybindings work from all contexts
- ✅ Search mode can be exited cleanly with Enter/Esc
- ✅ Switching between search modes works correctly
- ✅ Build successful: `cargo build` passes

---

## Impact

- **Severity**: New feature (enhances usability significantly)
- **Breaking changes**: None
- **Backward compatibility**: Fully maintained
- **Performance**: Minimal impact, filtering is efficient with fuzzy matching

---

## Related Issues

- Fixes [#116](https://github.com/FedericoBruzzone/tgt/issues/116): Chat Filter && Folder Support - Adds search bar functionality to filter through chats and messages

---

## Usage Examples

### Chat List Search
1. Press `Alt+R` (or focus ChatList and press `Alt+R`)
2. Type to filter chat names in real-time
3. Use arrow keys or Tab to navigate filtered results
4. Press Enter or Esc to exit search

### Message Search
1. Open a chat
2. Press `Alt+R` (or focus Chat and press `Alt+R`)
3. Type to filter messages in real-time
4. Use arrow keys or Tab to navigate filtered messages
5. Press Enter or Esc to exit search
6. Selected message remains selected when exiting search mode

### Quick Switching
1. While in message search mode, press `Alt+R` again
2. Instantly switches to chat list search
3. Can switch back and forth seamlessly
