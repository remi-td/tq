# Current bug and urgent issues

Created on 2026-01-18 at 20:15

## Tab Completion STTILL DOESN'T WORK PROPERLY

Completion doesn make sense AGAIN! See example: ![alt text](completion.png)
![alt text](image.png)

eg. tab after
  `select * from ` gives keywords when it should be databasenames, selecting a keyword inserts it at the beginning of the
  current line instead of where my cursor ar at, etc... These were right a few sprint ago!

## Export needs enhancements
Two key enhancements that I need you to prioritize:
- Export should allow to export to clipboard
- Export should allow to export ALL the dataset to a file: if I do a `select * from mytable;` you will limit the dataset to 100 (or the default) rows, which makes sense since we don't want to display all in the terminal not to pointlessly export all from the database. However, if I want to export to a file, I want to export ALL the dataset, not just the first 100 rows... Of course, if it's my who specified a limit (eg. `select top 1000 * from mytable;`), then it will export the 1000 rows to the file (this currently works fine). So what you need to so is just to make sure we export the full dataset when no limit is specified by the user.

## Branding
It's very sad but the tool has no logo, no welcome message at all. We need a bare minimum of brand identity when we start the tool. This was discussed in your specifications and still there is zero progress on this aspect. We need it so it can be presented to our clients and users.