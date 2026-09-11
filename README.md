# wikid

![crates.io](https://img.shields.io/crates/v/wikid.svg)
[![Rust](https://img.shields.io/badge/built_with-Rust-dca282.svg?logo=rust)](https://www.rust-lang.org/)

a feature-rich terminal wikipedia client.

## features

- **rich launch dashboard**: big ascii logo, live wikipedia metrics and a continue reading section.
- **tabs & splits**: work with multiple articles side-by-side or in tabs.
- **smart status bar**: 3-segment layout with active history trails and contextual action hints.
- **vim-like navigation**: intuitive keybindings for fast scrolling, jumping, heading traversal and pane management.
- **table of contents (`o`)**: pop-up outline modal with hierarchical section numbering.
- **zen mode (`z`)**: distraction-free reading canvas with no borders, tab bars, or status indicators.
- **personalized recommendation feed (`F`)**: article discovery feed tailored to your reading history. inspired by [xikipedia](https://github.com/rebane2001/xikipedia).
- **random article discovery (`r`)**: instantly fetch and explore random wikipedia articles in new tabs.
- **in-page search (`/`)**: exact substring search with live match highlighting and cycling (`n` / `N`).
- **inline article images (`I`)**: rich terminal graphics via Kitty graphics protocol with fallback to Unicode halfblocks.
- **custom saved lists (`m` / `M`)**: save articles into custom lists stored in `~/.config/wikid/saved_articles.json`.
- **link preview cards (`p` / `ctrl-hover`)**: peek at Wikipedia article summary cards.
- **spoken audio player & sleep timer (`a` / `:sleep`)**: stream [spoken](https://en.wikipedia.org/wiki/Wikipedia:WikiProject_Spoken_Wikipedia) Wikipedia articles with background caching, live progress bar, and configurable sleep timer.
- **live settings (`,`)**: in-app settings modal with instant hot-reloading.
- **instant qr code (`:qr`)**: scan terminal qr code to hand off reading to mobile.

## installation

### pre-built binaries

download standalone pre-compiled binaries for linux (x86_64), macos (apple silicon), and windows from [GitHub Releases](https://github.com/sharkthakftw/wikid/releases).

### from crates.io

```bash
cargo install wikid
```

### arch linux (via PKGBUILD)

```bash
git clone https://github.com/sharkthakftw/wikid.git
cd wikid
makepkg -si
```

### NetBSD (from the official repository)

```bash
pkgin install wikid
```

Or, if you prefer to build from source

```bash
cd /usr/pkgsrc/www/wikid
make install
```

### build from source
```bash
git clone https://github.com/sharkthakftw/wikid.git
cd wikid

# build and install the binary locally via cargo
make install
```

## keybindings

| action | keybinding | description |
| :--- | :---: | :--- |
| **scroll down / up** | `j` / `k` | scroll line down / up |
| **page down / up** | `f` / `b` | scroll full page down / up |
| **jump to top / bottom** | `g` / `G` | jump directly to article top / bottom |
| **history back / forward** | `H` / `L` | navigate backward / forward in article page history |
| **jump back / forward** | `ctrl-o` / `ctrl-i` | navigate backward / forward across intra-article jump history |
| **save to list** | `m` | save active article or item into custom list |
| **view saved lists** | `M` | open custom lists & articles viewer |
| **open recent article** | `1`–`7` | open recent article from launch dashboard |
| **search wikipedia** | `ctrl-s` | open search modal (opens in new tab) |
| **open search result** | `0`–`9` | open numbered search result directly |
| **edit search** | `i` | edit current search query in active tab |
| **in-page search** | `/` | in-page text search with match highlighting |
| **next / prev match** | `n` / `N` | jump to next / previous in-page search match |
| **table of contents** | `o` | open centered article outline modal |
| **zen mode** | `z` | toggle minimalist borderless reading view |
| **toggle images** | `I` | toggle rendering of inline article illustrations |
| **random article** | `r` | fetch & open random wikipedia article in new tab |
| **heading jump** | `]` / `[` | jump to next / previous section heading |
| **toggle feed** | `F` | toggle recommendation feed mode |
| **link navigation** | `tab` / `shift-tab` | focus next / previous article link |
| **preview link** | `p`, `ctrl-hover` | peek article summary card without navigating |
| **open link** | `enter` | open link in active pane |
| **open in new tab** | `t`, `alt-click` | open link in a new focused tab |
| **open in background tab** | `alt-enter`, `middle-click` | open link in a background tab |
| **open link in split** | `s` / `v` | open link in horizontal (`s`) or vertical (`v`) split |
| **copy link** | `y` | copy focused link to clipboard |
| **copy article URL** | `Y` | copy current article URL to clipboard |
| **split pane** | `ctrl-w` `s`/`v` | split active pane horizontally (`s`) or vertically (`v`) |
| **resize split** | `ctrl-=` / `ctrl--` | expand (`ctrl-=`) or shrink (`ctrl--`) active split dimensions |
| **navigate panes** | `ctrl-h/j/k/l`, `ctrl-w` `h/j/k/l` | switch focus between split panes |
| **move panes** | `ctrl-shift-h/j/k/l`, `ctrl-w` `H/J/K/L` | move active pane position in split tree |
| **close pane** | `x` | close active pane |
| **reopen closed** | `u` | reopen last closed tab or split pane |
| **new tab** | `alt-t` | create a new empty tab |
| **switch tabs** | `alt-h` / `alt-l` | switch to previous / next tab |
| **visit previous tab** | `%` | switch to last visited tab |
| **jump to tab** | `alt-0..9` | switch to tab 1-10 |
| **categories** | `c` | view categories of current article |
| **daily feeds (home)** | `f` / `n` / `d` / `t` | open featured article (`f`), news (`n`), on this day (`d`), trending (`t`) |
| **spoken audio** | `a` / `A` | play/pause / stop spoken audio |
| **seek audio** | `<` / `>` | scrub audio backward / forward 10s |
| **command palette** | `:` / `ctrl-p` | open sioyek-style command palette modal |
| **restore session** | `S` | restore previous session (from home tab) |
| **check for updates** | `U` | query github releases for latest tag |
| **settings modal** | `,` | open interactive settings modal |
| **help popup** | `?` | toggle keybindings cheat sheet |
| **quit** | `q` | exit wikid |

## configuration

wikid can be configured via `~/.config/wikid/config.toml` or via the settings modal from within wikid (`,`).

```toml
[general]
liked_readonly = true # when set to false, allow deleting items from the "Liked" list
auto_restore_session = false # when set to true, automatically restore previous session on launch
confirm_quit = true # enable/disable confirmation prompt on quit
hint_mode = "semantic" # continue reading hints: "semantic", "numbered", or "none"

[reader]
scroll_lines = 1 # number of lines to scroll per j/k press (1-20)
underline_links = false # enable/disable underlining links in articles
show_footnotes = true # enable/disable inline citation markers and references
show_external_links = true # enable/disable external links section
toc_section_numbers = true # enable/disable table of contents section markers
heading_marker = true # enable/disable colored heading marker in section headings
code_line_numbers = true # enable/disable line numbers in code blocks
show_images = true # enable/disable inline article images
image_protocol = "auto" # graphics protocol: "auto", "kitty", "halfblocks", "off"
halfblock_filter = "catmullrom" # resampling filter: "nearest", "triangle", "catmullrom", "gaussian", "lanczos3"
max_image_height = 25 # maximum terminal rows allocated per image

[ui]
rounded_borders = false # enable/disable rounded borders
icons = true # enable/disable icons
scroll_indicator = true # enable/disable scroll indicator on the right edge
stats = true # enable/disable live wikipedia statistics on launch screen
dim_inactive_panes = false # enable/disable dimming unfocused panes in multi-pane splits

[search]
limit = 20 # maximum number of search results returned from wikipedia (5-50)

[network]
timeout = 10 # network request timeout in seconds
offline_cache = true # enable/disable caching downloaded articles
cache_lifetime = 24 # cache expiration lifetime in hours (1-168)

[input]
mouse_support = true # enable/disable mouse clicks, tab switching, and scroll wheel
scroll_speed = 3 # number of lines to scroll per mouse wheel tick (1-20)
```

## cli usage

wikid can also be run headlessly from the command line without opening the tui:

| command | action |
| :--- | :--- |
| `wikid search <query>` | search wikipedia articles (`-l, --limit`, `-j, --json`) |
| `wikid summary <title>` | fetch and print article summary (`-j, --json`) |
| `wikid random` | fetch and print random article summary (`-f, --full`, `-j, --json`) |
| `wikid completions <shell>` | generate shell completions (`fish`) |
| `wikid help` | print help information |
| `wikid version` | print version |

## acknowledgements

- special thanks to [xikipedia](https://github.com/rebane2001/xikipedia) for inspiring the recommendation feed feature!
- thanks to [0323pin](https://github.com/0323pin) for packaging and maintaining the NetBSD package.

## license

distributed under the [MIT license](LICENSE).
