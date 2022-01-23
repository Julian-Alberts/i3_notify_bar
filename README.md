# i3_notify_bar

i3 notify bar is a simple notification bar for i3 window manager.

## Configuration

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

You can find more information [here](https://github.com/Julian-Alberts/i3_notify_bar/blob/master/config.md)