# Configuration

Example Bar definition:
~/.config/i3/config
```
bar {
    position top
    tray_output none
    workspace_buttons no
    status_command $HOME/.config/i3/i3_notify_bar $HOME/.config/i3/notify_rules.conf
}
```

Rule example:
```
def
  rule
    app_name = Spotify
  endrule
  action
    ignore
  endaction
enddef
```

A rule definition always starts with `def` and ends with `enddef`. A definition can contain a `rule`, an `action` and a `style` block.

### Blocks

#### rule

Rules start with a key and an equal sign (=) and end with a value it should be compared with.
example
```
rule
    app_name = Spotify
endrule
```

rule keys:
| name | values |
| --- | --- |
|app_name| application name |
app_icon| application icon |
summary | short notification |
body | notification text |
urgency | Urgency of this message. possible values are: low, normal, critical |
expire_timeout | timeout in secs. -1 this notification will not be hidden

In addition, summary and body support the match comparison method. This enables the matching of messages with regex.

example
```
rule
    body match Message from .*
endrule
```
#### action
Actions allow you to change the behavior of the bar. 

`ignore` Action

If the `ignore` action is specified the notification will not be displayed.

example:
```
action
   ignore
endaction
```

`stop` Action

If the `stop` action is specified no further rules will be validated.

`set` Action

The set Action is used to modify notification properties

allowed properties
| name | values |
| --- | --- |
|app_name| application name |
icon| application icon |
text | Display text. This option supports [templates](https://github.com/Julian-Alberts/i3_notify_bar/blob/master/template.md)|
expire_timeout | timeout in secs. If expire_timeout is set to -1 the message won't close automaticily.
emoji_mode | How emojis should be handled. Valid values: remove, replace, ignore

#### style
With the style block it is possible to change the appearance of a notification

example

```
style
  background #FF0000
endstyle

```

| name | values |
| --- | --- |
| background | RGB colors in HEX format
| text | RGB colors in HEX format

### Example
```
def
  rule
    app_name = test
  endrule
  action
    set text Test app notification
    set expire_timeout 10
  endaction
  style
    background #146D71
    text #cccccc
  endstyle
enddef
```
