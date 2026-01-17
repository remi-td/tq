# Interactive UI user stories

We would like an interactive UI that overall looks like state of the art interactive CLI UIs for database tools Ieg. https://github.com/dbcli/mycli, https://github.com/dbcli/pgcli, https://harlequin.sh/)... But better!

## Current bugs identified
The column names are not displayed! 

```
tq> select top 10 tablename, count(1) from dbc.tablesV group by 1;
╭───────────────────────────────────┬──────╮
│ col1                              ┆ col2 │
╞═══════════════════════════════════╪══════╡
│ U_0000C508_CF                     ┆    1 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┤
│ SessionStateTbl                   ┆    1 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┤
│ TD_ORDINALENCODINGFITMAP_C        ┆    1 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌┤
│ QryLogObjectsV                    ┆    1 │
```

Also, there is currently no limit in REPL mode.... even if the configuration isn't allowed yet, have a limit of eg 100 rows to make the UI usable...

## Features

- Syntax highlighting
- Auto-completion
- Command history
- Multi-line input
- Support for Teradata dialect
- Capability to save and load queries: saving queries should allow to save history as well if a certain depth or time interval is specified (ie. last 10 queries or last 10 minutes of queries).
- Capability to save resultsets (to file and clipboard)
- Capability to pan resultsets: scroll up and down for datasets larger than the terminal width/height
- Capability to open multiple sessions: ie. put current session in background and open a new one, switch back and forth between sessions.
- Identification on the system: on startup display a nice "tq" logo, welcome message, the database version number (from dbc.dbcinfo), the system name (from the logon string) and the current user name (from `select user).
- Needs to be beatiful, multiple standard CLI themes available, and a Teradata one with teradata orange color for accents and whatever goes well with it.
- Ascii art logo displayed on start on the left (right is system information discussed above): "tq" lower case with "t" in teradata orange and "q" in white/black.
- Should allow for special commands with `/` eg. `/systemspace`, `/databasespace`, `/monitor`, `/sessions`, `/health`, `/descibe`

## Technical requirement
- Needs to be as light as possible, extremely robust and super fast.
- Needs to be easy to install: should be available as a single binary (+ teradata driver library as license may be constraigning)
- Needs to be hackable: a lot can be specified in config files: editor behaviour and hotkeys (eg. vim, emacs...), row limits, default output format, user directory for history and saved queries, etc. That config file should come with reasonable defaults, an assistant should help in the user home directory (eg: `~/.tq/config.toml` on UNIX or whatever follows best practices). If the config file is found where the program is started, it should be used.