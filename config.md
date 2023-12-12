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

*Example:*
```
rule
  condition
    app_name = Spotify
  end
  action
    ignore
  end
end
```

A rule definition always starts with `rule` and ends with `end`. A definition can contain a `condition`, an `action`, and a `style` block.

## Blocks

### condition

Conditions start with a key followed by an operator and end with a value it should be compared with.
*Example:*
```
condition
    app_name = Spotify
end
```

The equals operator is available for all keys. Additionally `expire_timeout` supports <, <=, > and >=. 

Rule keys:
| name | values |
| --- | --- |
|app_name| application name |
app_icon| application icon |
summary | short notification |
body | notification text |
urgency | Urgency of this message. possible values are low, normal, critical |
expire_timeout | timeout in secs. If expire_timeout is -1, it will never expire.
group|This will be used to group similar notifications. By default, `group` is an empty string.

In addition, `summary` and `body` support the match comparison method. This enables the matching of messages with regex.

*Example:*
```
condition
    body match Message from .*
end
```
### action
Actions allow you to change the behavior of the bar. 

`ignore` Action.

The notification will not be displayed if the `ignore` action is specified.

*Example:*
```
action
   ignore
endaction
```

#### `stop` Action

No further rules will be validated if the `stop` action is specified.

#### `set` Action

The set Action is used to modify notification properties.

allowed properties
| name | values |
| --- | --- |
icon| application icon |
text | Display text. This option supports [templates](https://github.com/Julian-Alberts/i3_notify_bar/blob/master/template.md)|
expire_timeout | timeout in secs. If expire_timeout is set to -1, the message won't close automatically.
emoji_mode | How emojis should be handled. Valid values: remove, replace, ignore
group|Add notification to group. Groups are identified with a string.

#### style
With the style block, it is possible to change the appearance of a notification.

*Example*

```
style
  background #FF0000
end

```

| name | values |
| --- | --- |
| background | RGB colors in HEX format
| text | RGB colors in HEX format

*Example:*
```
rule
  condition
    app_name = test
  end
  action
    set text Test app notification
    set expire_timeout 10
  end
  style
    background #146D71
    text #cccccc
  end
end
```

### Sub `rule`
Subrules are only evaluated after all actions and styles of the outer rule are applied.
Subrules are evaluated in the same order as they are defined. 

*Example:*
```
rule
  condition
    app_name = test
  end
  action
    set text Test app notification
  end
  rule
    condition
      expire_timeout = -1
    end
    action
      set expire_timeout 10
    end
  end
end
```
