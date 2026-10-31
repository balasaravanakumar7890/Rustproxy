# UI Architecture & Ergonomics Analysis

This document specifies the user interface requirements, interaction patterns, layout mechanics, and component hierarchy for `RustProxy`, modeled after modern, high-efficiency desktop security testing environments.

---

## 1. Design Principles & Aesthetic Foundations

1. **Information Density**: Security analysis involves examining hundreds of headers, query parameters, and multi-megabyte payloads. Screen real estate must prioritize content over decoration.
2. **Dark Theme Ergonomics**: Professional dark palette (`#121316` surface, `#1a1c23` elevated cards, `#2a2d3d` borders, `#e2e8f0` high-contrast typography, `#3b82f6` accent blue, `#10b981` success green, `#f59e0b` warning amber, `#ef4444` error red).
3. **Low-Latency Virtualization**: Large traffic histories (50,000+ items) must scroll at 60 FPS without DOM thrashing using virtual list rendering.
4. **Keyboard-First Workflow**: Critical actions (`Forward`, `Drop`, `Send to Repeater`, `Send Request`) must be accessible via global and context-sensitive key shortcuts.

---

## 2. Main Navigation Structure

The top-level shell features a primary horizontal navigation bar with sub-tabs for active tools:

```
+-----------------------------------------------------------------------------------------+
| [RustProxy v0.1.0]   Proxy | Repeater | Target | Scanner | Decoder | Comparer | Settings  |
+-----------------------------------------------------------------------------------------+
| Sub-tabs: [ Intercept ]  [ HTTP History ]  [ WebSocket History ]  [ Proxy Options ]      |
+-----------------------------------------------------------------------------------------+
| Toolbar:  [Intercept is ON / OFF]  [Forward]  [Drop]  [Filter: Show all]                |
+-----------------------------------------------------------------------------------------+
| Workspace Main Panel (Split Pane / Tabbed views)                                        |
|                                                                                         |
|                                                                                         |
+-----------------------------------------------------------------------------------------+
| Status Bar: Port 8080 Active | Intercepted: 1 | Flows: 1,420 | Memory: 42 MB            |
+-----------------------------------------------------------------------------------------+
```

---

## 3. Core Component Layouts

### 3.1 Proxy Intercept View
- **Controls**:
  - `Intercept is on/off` toggle switch with vibrant color indicators.
  - `Forward` (Ctrl+F) sends the pending item.
  - `Drop` (Ctrl+D) cancels and terminates the pending item.
  - Dropdown: `Action` -> `Send to Repeater (Ctrl+R)`, `Send to Comparer`, `Add Host to Scope`.
- **Split Editor Layout**:
  - Upper: Editable request text area with syntax highlighting.
  - Form tabs: `Raw`, `Headers`, `Params`, `Hex`.

### 3.2 HTTP History View
- **Top Filter Bar**:
  - Fast search query bar (regex, text, status code).
  - Scope toggle: `[x] Show only in-scope items`.
  - Type filters: `[ ] 2xx [ ] 3xx [ ] 4xx [ ] 5xx | MIME: HTML, JSON, Scripts, Other`.
- **Flow Table (Virtualized)**:
  - Columns: `#`, `Time`, `Method`, `Host`, `Path`, `Status`, `Length`, `MIME`, `Duration`, `Comment`.
  - Right-click Context Menu:
    - `Send to Repeater`
    - `Send to Comparer (Request / Response)`
    - `Copy URL / Copy as cURL / Copy Raw`
    - `Add / Remove from Scope`
    - `Highlight (Colors)`
- **Detail Split Pane (Bottom or Right)**:
  - Resizable divider.
  - Left child: Request viewer (Raw, Headers, JSON, Hex).
  - Right child: Response viewer (Raw, Headers, Preview, JSON, Hex).

### 3.3 Repeater View
- **Tab Bar**: Horizontal scrollable tabs with close `x`, double-click to rename, right-click `Duplicate Tab`.
- **Split Pane**: Request on left, Response on right.
- **Header Bar**:
  - `Send` button (large, prominent, `Ctrl+Enter`).
  - Target URL selector `https://api.example.com:443`.
  - Follow redirects checkbox.
  - Response metrics badge: `Status: 200 OK | Time: 42 ms | Size: 1,234 B`.

### 3.4 Target & Sitemap View
- **Dual Pane**:
  - Left pane: Tree view of targets organized by `Host` -> `Directory` -> `Resource`.
  - Right pane: List of requests associated with the selected node + Request/Response preview.

---

## 4. Keyboard Shortcuts Reference Table

| Shortcut | Scope | Action |
|---|---|---|
| `Ctrl + R` | History / Intercept | Send selected request to Repeater |
| `Ctrl + Enter` | Repeater | Send active request |
| `Ctrl + F` | Intercept | Forward pending intercepted message |
| `Ctrl + D` | Intercept | Drop pending intercepted message |
| `Ctrl + T` | Repeater | Open new empty Repeater tab |
| `Ctrl + W` | Repeater | Close current tab |
| `Ctrl + K` | Global | Focus filter / search bar |
| `Ctrl + Shift + C` | History | Copy request as cURL command |
