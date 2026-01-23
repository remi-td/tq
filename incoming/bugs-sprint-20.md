## Tab Completion 
First of all congratulations for fixing it after 10 sprints!

Make sure that you know how to test this for regression automatically.


There's still a few odd things:

### Missing dbc database!

If I do `sel * from `+TAB I get a list of many databases, it should contain all databases on the system, but I noticed that I am using the dbc one!!! 

Make sure all databses are included
```
tq> | sel * from d
demo_user          (database)
DemoNow_Monitor    (database)
```

### Tab behaviour
What we hit tab the first time, the object menu is displayed, which is OK.
But when we hit tab a second time, the dursor select the next object (down) which is unintuitive (the down arrow is for this), typically a second tab hit validates the completion with the highlighted object (same as enter).
Also, when I hit tab on a database after a FROM/JOIN, I would expect to complete the database name, add a '.' and prompt the list of tables in this database directly.

### Databases objects not cached/fetched

Some databases objects are not cached/fetched

For example:
```
tq> | sel * from demo_user.
NO RECORDS FOUND
```
--> I know that there are three tables in this database, but it should be fetched!

Others seem to work fine:
```
tq> | sel * from modelops.
modelops.aoa_byom_models             modelops.aoa_byom_models (table)
modelops.aoa_statistics_metadata     modelops.aoa_statistics_metadata (table)
modelops.pima_patient_diagnoses      modelops.pima_patient_diagnoses (table)
modelops.pima_patient_features       modelops.pima_patient_features (table)
modelops.pima_patient_predictions    modelops.pima_patient_predictions (table)
```