# UI modes

Switchable browser UI modes: the toolbar and status bar change to the
layout of other 1990s browsers, and to each version whose interface changed
significantly. This folder tracks the work.

- [spec.md](spec.md): how modes work (architecture, naming, testing).
- [modes.md](modes.md): every planned mode's layout, and what it's
  modelled on.
- Mockups: the design canvas linked from [modes.md](modes.md).
- Research: [`docs/research/90s-browsers/`](../research/90s-browsers/).

Update the tables below in the same commit as the work they describe.

## Status

| Step | What | Status |
|---|---|---|
| 1. Shared controller | `ui/chrome/chrome.js`; Lantern's pages move to `ui/chrome/lantern/`, unchanged on screen | Done |
| 2. Branding rule | Faithful layouts and ordinary wording allowed; names and artwork never (steward skill) | Done |
| 3. Mode setting | `UiMode` in `settings.rs`, saved as `ui_mode` | Done |
| 4. Switching | `--ui=`, runtime switch and relayout, the lockdown change, native menus (`menu` command), Browser UI submenu, Ctrl+Shift+M | Done |
| 5. First mode | `workstation-1993`, with contract tests and goldens | Done |
| 5b. Page styles | Each mode's browser defaults under pages, and its era look on request (`mode_pages`) | Done |
| 6. Settings page | A Browser UI section; needs the maintainer to approve new settings goldens | Not started |
| 7. Layout extras | Hidden views, chrome below the page, inset border, content zoom (the TV modes) | Not started |
| 8. More modes | See below; next: `toolbar-1996`, `desktop-1996`, `applet-1997` | Not started |

## Modes

| Key | Name | Modelled on | Needs | Status |
|---|---|---|---|---|
| `lantern` | Lantern (default) | — | — | Shipped |
| `workstation-1993` | Workstation (1993) | NCSA Mosaic for X 1.x–2.6 | menus | Built (not yet merged) |
| `workstation-1996` | Workstation (1996) | NCSA Mosaic for X 2.7 | icons | Planned |
| `workstation-pc-1995` | Workstation, PC (1995) | NCSA Mosaic for Windows 2 | icons | Planned |
| `desktop-1995` | Desktop (1995) | Internet Explorer 2 | icons | Planned |
| `desktop-1996` | Desktop (1996) | Internet Explorer 3 | icons, hover colour | Planned |
| `desktop-1997` | Desktop (1997) | Internet Explorer 4 | icons | Planned |
| `desktop-1999` | Desktop (1999) | Internet Explorer 5 | icons | Planned |
| `toolbar-1995` | Toolbar (1995) | Netscape 1.1–1.22 | icons | Planned |
| `toolbar-1996` | Toolbar (1996) | Netscape 2–3 | icons | Planned |
| `suite-1997` | Suite (1997) | Netscape Communicator 4 | icons | Planned |
| `classic-1992` | Classic desktop (1992) | MacWWW | menus, research | Planned |
| `classic-1994` | Classic desktop (1994) | NCSA Mosaic for the Mac | icons, research | Planned |
| `classic-1999` | Classic desktop (1999) | iCab | icons | Planned |
| `applet-1995` | Applet browser (1995) | HotJava alpha | icons, research | Planned |
| `applet-1997` | Applet browser (1997) | HotJava 1.x–3 | icons | Planned |
| `tv-1996` | Set-top TV (1996) | WebTV | layout extras | Planned |
| `tv-1997` | Set-top TV (1997) | WebTV | layout extras | Planned |
| `pc-1993` | Early PC (1993) | Cello | menus | Planned |
| `terminal-1992` | Text terminal (1992) | Lynx | monospace chrome | Later |
| `dos-1996` | DOS graphics (1996) | Arachne | a side view | Later |
| `cube-1990` | Cube workstation (1990) | WorldWideWeb | floating windows | Not planned |
| `workspace-1997` | Workspace (1997) | Opera 3 | several content views | Not planned |

Product names are trademarks of their owners. Lantern isn't affiliated with
or endorsed by them.
