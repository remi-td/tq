# Current bug and urgent issues

Created on 2026-01-18 at 17:00

## Tab Completion doesn't work properly

Completion doesn make sense AGAIN! See example: ![alt text](completion.png)

## Table display is broken
Broken AGAIN with the padding!!! Please stop the padding for now and pospone it for much later as it just breaks everthing. When you will be ready for padding, you will need to do some extensive research in how to do it right and test it very carefully.
Screenshot: docs/builder/incoming/table display-bug.png

This padding story isn't so important after all... I think that the key thing, for now, will be to recognize that the terminal has a specific width and that considering that width only a certain number of columns should be displayed for readibility... You can use the last few characters to add a last column stating | (+n cols) | in the table header and | ... | for the  body.
This will do for a long time. To implement an actual padding feature, we will need to do some extensive research in how to do it right and test it very carefully, most importantly you will need a test framework that will enable you to "see" like the user.... 