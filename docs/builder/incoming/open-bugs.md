# Current bug and urgent issues

Created on 2026-01-18 at 14:13

## Tab Completion doesn't work properly

Multiple error on auto completion:
- when typing `select * from <tab>`: only 9 databases are dosplayed and scrolling though it loops through these 9... there are many more!!!
- Multi-line doesn't work: if I do `select * from <dataabse-name>.`, hit return, then hit tab, the suggestions are the list of SQL keywords! It should be the tables in this database! See example: ![alt text](image.png)

## Wharnings and errors when opening the app:
I get the following errors as I open the application. Maybe this relates to the error above...
```
warning: tq@1.3.0: Successfully copied teradatasql.dylib to /Users/remi.turpaud/Code/genAI/tq/target/debug/teradatasql.dylib
warning: unused imports: `PagerConfig`, `display_with_pager`, and `should_page`
  --> src/commands/repl/executor.rs:11:20
   |
11 | use super::pager::{display_with_pager, should_page, PagerConfig};
   |                    ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `TableInfo`
  --> src/commands/repl/metadata_completer.rs:18:60
   |
18 | use crate::db::{ColumnInfo, DatabaseClient, MetadataCache, TableInfo};
   |                                                            ^^^^^^^^^

warning: `tq` (lib) generated 2 warnings (run `cargo fix --lib -p tq` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.59s
     Running `target/debug/tq repl`
```

# Error message formatting

Below is what an error messahe looks like: we get the full stack trace, which is not very user-friendly. Only the actual SQL error message is relevant and should be displayed.

```
tq> sel * from DemoNow_Monitor.
...> AS ;

Error: SQL syntax error: [Version 20.0.49] [Session 1429] [Teradata Database] [Error 3707] Syntax error, expected something like an 'UDFCALLNAME' keyword between '.' and the 'AS' keyword.
 at gosqldriver/teradatasql.formatError ErrorUtil.go:101
 at gosqldriver/teradatasql.(*teradataConnection).formatDatabaseError ErrorUtil.go:210
 at gosqldriver/teradatasql.(*teradataConnection).makeChainedDatabaseError ErrorUtil.go:226
 at gosqldriver/teradatasql.(*teradataConnection).processErrorParcel TeradataConnection.go:347
 at gosqldriver/teradatasql.(*TeradataRows).processResponseBundle TeradataRows.go:2724
 at gosqldriver/teradatasql.(*TeradataRows).executeSQLRequest TeradataRows.go:1194
 at gosqldriver/teradatasql.newTeradataRows TeradataRows.go:805
 at gosqldriver/teradatasql.(*teradataStatement).QueryContext TeradataStatement.go:122
 at gosqldriver/teradatasql.(*teradataConnection).QueryContext TeradataConnection.go:836
 at database/sql.ctxDriverQuery ctxutil.go:48
 at database/sql.(*DB).queryDC.func1 sql.go:1786
 at database/sql.withLock sql.go:3572
 at database/sql.(*DB).queryDC sql.go:1781
 at database/sql.(*Conn).QueryContext sql.go:2037
 at main.createRows goside.go:1142
 at main.rustgoCreateRows goside.go:999
 at _cgoexp_c43d071e9719_rustgoCreateRows _cgo_gotypes.go:416
 at runtime.cgocallbackg1 cgocall.go:446
 at runtime.cgocallbackg cgocall.go:350
 at runtime.cgocallback asm_arm64.s:1180
 at runtime.goexit asm_arm64.s:1268

tq> 
```