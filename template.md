# Template
For templates a custom template engine is used. The syntax is based on smarty. 

## Accessing Variables
To access a variable, simply surround its name with curly brackets.

Example:
```
{ app_name }
```

## Available Variables
| name | values | type |
| --- | --- | ---
app_name| application name | string
icon| application icon | string
body | Notification message | string
summary | Short message | string
expire_timeout | time in secs until the message is closed | number

## Datatypes

|name| description | Literal examples
|--|--|--
String| Contains text | "Foo"; "bar"
Number| floating point numbers| 10; 0.9  
Boolean| true or false (Is currently not used) | true; false

## Modifier

Modifiers allow to change the output of a variable. It is allowed to chain multiple modifiers. They will be executed left to right. Paramerters can be literals and variables.

The currently supportet modifiers are:


|Name|parameter|Description
|----|---|--
slice|start: number, length: number | Get slice of string 
regex||depricated use match instead
match|regex: String, group: Number = 0| Returns the matched group. Zero matches the entire regex.
replace|needle: String, to: String, count: Number = 0| Replaces string with another string. If count is 0 all occurrences will be replaced.
replace_regex|needle: String, to: String, count: Number = 0| Replaces regex match with a string. If count is 0 all occurrences will be replaced.
upper| | String to upper case
lower| | String to lower case
repeat| n: Number | Repeat String n times

Example:
```
{body|replace:"FOO":summary}
``` 
