# 2.0.0
Added max_len modifier
Refactored configuration
Improved stability
Lots of internal improvements

## Configuration
### Syntax
In previus versions sections used blocks with the following syntax:
```
<sectionname>
end<sectionname>
```
Starting with version 2.0.0 this has been simplified to
```
<sectionname>
end
```

A very basic configuration file might look something like this
```
rule
  condition
    expire_timeout > 30
  end
  action
    set expire_timeout 30
  end
end
```

### Compare
Comparison operators now depend on the type of the property and not the property itself.

Numbers: <, <=, =, >=, >
Strings: =, match
Urgency: Urgency levels are treated as numbers. low < normal < critical

### Groups
Groups are a new feature that allow the grouping of one or more notifications into one block.
This can be useful if an application creates lots of notifications.
This also adds a new `group` section to the configuration. This allows to set the colors of a group.
It behaves the same as a notification style section.
A `group` section can eigther apply to all or a single group.
```
rule
  action
    set group group-name
  end
end

group
  style
    background #000000
  end
end

group group-name
  style
    text #FF00FF
  end
end
```

### Notification sounds
To use notification sounds i3_notify_bar needs to be compiled with the `audio` feature.
Notifications sounds can be set like all other properties.
The volume can be set as a number between 0 and 100.
```
rule
  action
    set notification_sound <sound-file>
    set volume 1
  end
end
```
Notification sounds can be muted in the top right menu.


## 1.7.2
Updated clap
Improved error handling
Updated mini_template
Added == > >= < <= Operators for expire_timeout in conditions
Added date_time modifier for time
Performance improvements
