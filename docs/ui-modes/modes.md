# UI modes: the modes

One section per mode: what it's modelled on, what goes in the toolbar view
(above the page) and the status view (below it), and its menus. Keys and
names follow the branding rule (era and kind of machine, never a product).
Layouts follow the originals; all icons and the activity box's lantern are
original art. Where a layout is from memory rather than a screenshot in
[`docs/research/90s-browsers/`](../research/90s-browsers/), the section says
so: check it against a source before building.

The mockups for every mode are on the design canvas "Lantern UI mode comps"
(a private claude.ai artifact; ask the maintainer for access):
https://claude.ai/artifact/PKpwbxGtr3gGZNN5SRzLTH

Product names are trademarks of their owners. Lantern isn't affiliated with
or endorsed by them.

Heights are the toolbar and status views' heights in DIP. Planned ones may
change when the mode is built.

**Pages** gives each mode's browser defaults (for plain pages with no style
sheets of their own; [spec.md](spec.md#pages-under-a-mode)) and its era
look (a full retro look, when the user asks for it). Defaults are from
memory unless noted: check them against a source before building.

## Lantern

### `lantern`: Lantern (default)
Lantern's own UI. Toolbar 112: big icon-and-label buttons, Retro button,
lantern throbber; URL row; Quick links row. Status 24: text, modem pane,
progress meter, padlock.

## Workstation family

### `workstation-1993`: Workstation (1993)
Modelled on NCSA Mosaic for X 1.x–2.6 (screenshot in the catalog).
- **Toolbar** (~104): menu bar (File, Navigate, Options, Help; Help at the
  right); "Document Title:" and "Document URL:" fields; the lantern activity
  box at the right of both fields.
- **Status** (~72): status line (text, modem, meter, padlock); a row of text
  buttons: Back, Forward, Home, Reload, Stop, Open..., Bookmarks, Mark.
- Motif bevels, grey `#b3b3b3`, navy labels, Helvetica bold, Courier fields.
- Left out: Annotate, Documents and Manuals menus, Search Keyword, Clone,
  New Window, Close Window (no matching feature).
- **Pages**: grey `#bfbfbf` page, Times, black text (screenshot in the
  catalog). Era look: Early web (1995).

### `workstation-1996`: Workstation (1996)
Modelled on NCSA Mosaic for X 2.7 (from memory). As 1993, but an icon
toolbar (Back, Forward, Home, Reload, Open, Save, Clone, New, Close,
Hotlist) replaces the bottom buttons, and the status view becomes a meter,
status text and Stop.

### `workstation-pc-1995`: Workstation, PC (1995)
Modelled on NCSA Mosaic for Windows 2.0 (from memory). Menu bar; one row of
small icon buttons; URL combo; activity box at the right; status bar with
message, progress bar, NUM and clock panes.
**Pages**: grey page, Times. Era look: Early web (1995).

## Desktop family

### `desktop-1995`: Desktop (1995)
Modelled on Internet Explorer 2 (from memory). Menu bar; one toolbar of
small icon-only buttons (Open, Print, Send | Back, Forward, Stop, Refresh,
Home | Search, News, Favorites, Add | larger/smaller text | Cut, Copy,
Paste); Address row; activity box spanning both rows. Status: text,
progress, icon pane.

### `desktop-1996`: Desktop (1996)
Modelled on Internet Explorer 3 (from memory). Menu bar; bands with grip
handles: large labelled grey-until-hover buttons (Back, Forward, Stop,
Refresh, Home, Search, Favorites, Print, Font, Mail ▾); Address band and
Links band; tall activity box at the right of the bands.
**Pages** (whole Desktop family): white page, Times, blue and purple links.
Era look: none yet (a "Desktop web (1996)" look: tables, MS Sans Serif
forms, no CSS3).

### `desktop-1997`: Desktop (1997)
Modelled on Internet Explorer 4 (from memory). The menu bar is a band;
flat buttons with dropdown arrows on Back and Forward; Search, Favorites,
History, Channels, Fullscreen, Mail ▾, Print, Edit; Address and Links
bands; status bar with a page icon, progress and a zone pane.

### `desktop-1999`: Desktop (1999)
Modelled on Internet Explorer 5 (from memory). As 1997 with a small
activity box in the menu band, Tools menu, a Go button after the address,
and Discuss.

## Toolbar and Suite family

### `toolbar-1995`: Toolbar (1995)
Modelled on Netscape 1.1–1.22 (screenshot in the catalog). Menu bar (File,
Edit, View, Go, Bookmarks, Options, Directory; Help at the right); grouped
icon-only buttons (Back, Forward, Home | Reload, Images, Open, Print, Find |
Stop); Location combo; directory buttons (What's New!, What's Cool!,
Handbook, Net Search, Net Directory, Newsgroups); activity box spanning
Location and directory rows.
**Pages** (Toolbar 1995 and 1996): grey `#c0c0c0` page, Times, blue and
purple links (screenshots in the catalog). Era look: Early web (1995). Status: key pane,
"Document: Done", progress.

### `toolbar-1996`: Toolbar (1996)
Modelled on Netscape 2–3 (screenshot in the catalog). Small icons with
labels (Back, Forward, Home | Edit | Reload, Images, Open, Print, Find |
Stop); Location with a bookmark icon; directory buttons (What's New?,
What's Cool?, Destinations, Net Search, People, Software). Status adds mail
and help panes.

### `suite-1997`: Suite (1997)
Modelled on Netscape Communicator 4 (screenshot in the catalog). Three
toolbars with collapse tabs: navigation (Back, Forward, Reload, Home,
Search, Guide, Print, Security, Stop, activity box), location (Bookmarks
button, Location field, What's Related), personal toolbar (quick links).
Status: padlock, text, progress, a component bar of four small icons.
**Pages**: white page by then, Times. Era look: Web portal (1997).

## Classic desktop family

### `classic-1992`: Classic desktop (1992)
Modelled on MacWWW (from memory; no screenshot yet). One-bit black and
white, almost no toolbar: a URL strip under the title; navigation lives in
the menus (File, Edit, Navigate, Bookmarks).
**Pages**: white page, black text, Times or New York; images may need a
one-bit look. Era look: Early web (1995), or a new one-bit look.

### `classic-1994`: Classic desktop (1994)
Modelled on NCSA Mosaic for the Mac (from memory). Grey bevelled rounded
buttons with icons and labels, URL field, activity box; the status line
shares the bottom edge with the horizontal scroll bar.

### `classic-1999`: Classic desktop (1999)
Modelled on iCab (from memory). Late-90s grey look; big colour icon
buttons (Back, Forward, Home, Reload, Stop | Search, Hotlist, Images,
Print, Download); a face in the activity box that smiles for valid HTML;
URL row; status text and a striped progress bar.

## Applet browser family

### `applet-1995`: Applet browser (1995)
Modelled on the HotJava alpha (from memory; uncertain). Menu bar; Document
URL field; page; a bottom row of icon-only buttons (Back, Forward, Home,
Reload, Stop, Help), status text and the activity box.

### `applet-1997`: Applet browser (1997)
Modelled on HotJava 1.x–3 (screenshot in the catalog). Menu bar; icon
buttons with small labels (Back, Forward, Home, Reload, Stop, Search,
Print); activity box at the right; Address row; plain status bar.
**Pages** (both Applet modes): white page, sans-serif text (Dialog), as in
the catalog screenshot. Era look: none yet.

## Set-top TV family

These need the layout extras: chrome below the page, an inset border,
content zoom and arrow-key link navigation with a highlight box.

### `tv-1996`: Set-top TV (1996)
Modelled on WebTV (from memory). No toolbar: the page fills the screen,
the selected link has a yellow highlight box, and a bar along the bottom
shows the page title and a progress bar.
**Pages** (both TV modes): large type, a light grey-blue page, links shown
by the highlight box rather than underlines. Era look: a new "Set-top TV"
look (large text, single column, no small fonts), together with the layout
extras.

### `tv-1997`: Set-top TV (1997)
Modelled on WebTV (from memory). As 1996, with the Options panel open over
the bottom of the page: Go to, Find, Info, Save, Send, Reload, Text size,
Print, Browser UI, Setup.

## More families

### `cube-1990`: Cube workstation (1990)
Modelled on WorldWideWeb (screenshot in the catalog). Needs floating
windows: a vertical menu at the top left (Info, Navigate, Document, Find,
Edit, Links, Style, Print, Windows, Services, Hide, Quit), a Navigation
panel (< Previous, Back up, Next >, Home, Help) and a document window with
the scroller on the left. Probably needs a frameless window; not planned.

### `terminal-1992`: Text terminal (1992)
Modelled on Lynx (screenshot in the catalog). An 80-column terminal: title
at the top right, highlighted links, a "press space for next page" bar and
two lines of key help.
**Pages**: black page, light grey monospace text, bright links. This mode
only works with its era look, so `era-look` should be its default: a new
"Text terminal" retro look (monospace, no images but their alt text,
highlighted links, everything in one column).

### `pc-1993`: Early PC (1993)
Modelled on Cello (from memory). A Windows 3.1 frame; menu bar (File, Edit,
Jump, Bookmark, Search, Configure, Help); no toolbar; status line with a
Ready pane and the URL.

### `dos-1996`: DOS graphics (1996)
Modelled on Arachne (screenshot in the catalog). Title and URL rows on top;
a right-hand panel with a big activity box, a 3×3 grid of icon buttons
(Back, Forward, Home, Reload, Add, Bookmarks, Stop, Search, Help), a list
of shortcuts and a system-stats box. Needs a side view.
**Pages**: white page, Arial-like text. Era look: Early web (1995).

### `workspace-1997`: Workspace (1997)
Modelled on Opera 3 (from memory). Several page windows inside one main
window, a Hotlist side panel and a zoom box. Needs multiple content views;
not planned.
