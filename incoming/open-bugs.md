# Current bug and urgent issues

Updated on 2026-01-23 at 15:15

## New logo design
**New Design (LOWERCASE):**
Let's try this logo (still witht the t in orange - (RGB ≈ 255,95,0)): 

```
 __                
/\ \__             
\ \ ,_\    __      
 \ \ \/  /'__`\    
  \ \ \_/\ \L\ \   
   \ \__\ \___, \  
    \/__/\/___/\ \ 
              \ \_\
               \/_/
```   


This is a lowercase 't' (left) in Teradata orange and lowercase 'q' (right) in default color, using block characters for clarity.

## BUG 2: Tab Completion Shows Pager Output
We still have the same issue: If I press tab after `select * from ` I get:
```
tq> ? select * from 
Page 1: records 0 - 0  total: 0  
```
You story about teradatarustapi is writing directly to TTY doesn't make any sense to me since the query functionality works well otherwise and uses the same drivers...

What you need to do:
- Cache all database names at startup (`sel databasename from dbc.databases;`).
- Cache all database object names incrementally as the databases are used (ie. I require completion after a `...<databasename>.`) (`sel tablename from dbc.tablesV where databasename = <databasename>;`).
- Whenever you detect a completion request after a FROM/JOIN keyword, pop the menu of databases, filter as I start typing, allow up/down navigation, autocomplete the `.` and pop the menu of objects in the selected database, etc...

What I strongly recommend you do:
- Research on how this is best implemented in other tools built in Rust
- Direct some effort understanding the current issue and producing a robust design
- Ensure that you have a test mechanism.