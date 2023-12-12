# Template
For templates, a custom template engine is used. The syntax is based on smarty. 

## Accessing Variables
To access a variable, surround its name with two curly brackets.

Example:
```
{{ app_name }}
```

## Available Variables
| name | values | type |
| --- | --- | ---
app_name| application name | string
icon| application icon | string
body | Notification message | string
summary | Short message | string
expire_timeout | time in secs until the message is closed | number
time | time when the message was received in seconds since 1970.01.01 | number

## Datatypes

|name| description | Literal examples
|--|--|--
String| Contains text | "Foo"; "bar"
Number| floating point numbers| 10; 0.9  
Boolean| true or false (Is currently not used) | true; false

## Modifier

Modifiers allow the output of a variable to be changed. It is allowed to chain multiple modifiers. They will be executed left to right. Parameters can be literals and variables.

The currently supported modifiers are:


|Name|parameter|Description
|----|---|--
slice|start: number, length: number | Get slice of string 
regex||depricated use match instead
match|regex: String, group: Number = 0| Returns the matched group. Zero matches the entire regex.
replace|needle: String, to: String, count: Number = 0| Replaces string with another string. If the count is 0, all occurrences will be replaced.
replace_regex|needle: String, to: String, count: Number = 0| Replaces regex match with a string. If count is 0 all occurrences will be replaced.
upper| | String to upper case
lower| | String to lower case
repeat| n: Number | Repeat String n times
date_time | format_string: String | Format number as date; You can find more information in the [chrono documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)

Example:
```
{{ body|replace:"FOO":summary }}
``` 
