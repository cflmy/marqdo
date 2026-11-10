# Article form data

The callable contract is forms.mq.md. These are literal records, not another program or a second submission implementation. Fields map to the existing articles schema; native rendering and server validation consume the same records.

## fields

Title and slug are required. Summary and body remain optional, with their original input types.

| @ | name | label | type | required |
|---|---|---|---|---|
| 1 | title | Title | text | True |
| 2 | slug | Slug | text | True |
| 3 | summary | Summary | textarea | False |
| 4 | body | Body | textarea | False |

## rules

These are the original server rules and error messages; browser hints do not replace them.

| @ | field | rule | message |
|---|---|---|---|
| 1 | title | required | Title is required |
| 2 | title | max:120 | Title is too long |
| 3 | slug | required | Slug is required |
| 4 | slug | max:80 | Slug is too long |
| 5 | summary | max:500 | Summary is too long |
| 6 | body | max:8000 | Body is too long |
