# Current bug and urgent issues

Created on 2026-01-18 at 20:15

## Logo and branding

You missed your own branding guidelines: the `tq` LOGO should be written in lowercase with th 't' in the Teradata orabge color (#F37021). 

I would also use the same teradata orange as default for the interactive prompt color: `tq>` (currently olive green).
The problem is that some terminals do not support truecolor (24-bit)....
So let's use the closest xterm-256 match to #F37021 is typically color index 202 (RGB ≈ 255,95,0). That’s not perfect (ΔR +12, ΔG −17, ΔB −33), but it will look similar to teradata orange. 

## Tab Completion STTILL DOESN'T WORK PROPERLY

This is the third sprint where you failed to implement tab completion properly.![alt text](completion.png)
![alt text](image.png)

This does't work on special words either. eg. `sel * fr`+tab gives me all possible reserved words! (obviously this should automatically complete with `FROM` as this is the only possible word).
eg. tab after
  `select * from ` gives keywords when it should be databasenames, selecting a keyword inserts it at the beginning of the
  current line instead of where my cursor ar at, etc... These were right a few sprint ago!

This has exactly the same issue as last sprint, and the two before that...

## Export needs enhancements
Two key enhancements that I need you to prioritize:
- This is confusing:
```
tq> /export
Usage: /export <format> [file]
       /export <format> clipboard
       /export clipboard [format]
       /export <format> --append [file]

Formats: table (default), csv, json, sql
Examples:
  /export csv results.csv
  /export clipboard csv
  /export json clipboard
  ```
  and from /help: 
  ```
    /export <fmt> [file]   Export last result (table, csv, json, sql)
    /export clipboard      Copy last result to clipboard
```

Make it simply /export <format> [file|clipboard] so semantics are clear!

- THIS STILL DOESN'T WORK PROPERLY: Export should allow to export ALL the dataset to a file: if I do a `select * from mytable;` you will limit the dataset to 100 (or the default) rows, which makes sense since we don't want to display all in the terminal not to pointlessly export all from the database. However, if I want to export to a file, I want to export ALL the dataset, not just the first 100 rows... Of course, if it's my who specified a limit (eg. `select top 1000 * from mytable;`), then it will export the 1000 rows to the file (this currently works fine). So what you need to so is just to make sure we export the full dataset when no limit is specified by the user.

## Warning messages when starting
I see this bunch of warnings when sarting: definitely doesn't look professionnal at all, and I DON'T UNDERSTAND WHY YOU HAVE NOT TESTED THAT - THIS IS NOT ACCEPTABLE IN A FINAL PRODUCT!!!

(base) remi.turpaud@TD-VX1J6LT4PX tq % cargo run --  repl
warning: tq@1.6.1: Successfully copied teradatasql.dylib to /Users/remi.turpaud/Code/genAI/tq/target/debug/teradatasql.dylib
warning: unused `std::result::Result` that must be used
   --> src/commands/repl/mod.rs:239:5
    |
239 |     writeln!(writer, "{}", orange.paint("  _____"));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled
    = note: `#[warn(unused_must_use)]` on by default
    = note: this warning originates in the macro `writeln` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unused `std::result::Result` that must be used
   --> src/commands/repl/mod.rs:240:5
    |
240 |     writeln!(writer, "{}", orange.paint(" |_   _|__ _"));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled
    = note: this warning originates in the macro `writeln` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unused `std::result::Result` that must be used
   --> src/commands/repl/mod.rs:241:5
    |
241 |     writeln!(writer, "{}", orange.paint("   | |/ _` |"));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled
    = note: this warning originates in the macro `writeln` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unused `std::result::Result` that must be used
   --> src/commands/repl/mod.rs:242:5
    |
242 |     writeln!(writer, "{}", orange.paint("   | | (_| |"));
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: this `Result` may be an `Err` variant, which should be handled
    = note: this warning originates in the macro `writeln` (in Nightly builds, run with -Z macro-backtrace for more info)

warning: `tq` (lib) generated 4 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.76s
     Running `target/debug/tq repl`