# i3_notify_bar

The i3 notify bar is a simple notification bar for the i3 window manager.

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
rule
  condition
    app_name = Spotify
  end
  action
    ignore
  end
end
```

A rule always starts with `rule` and ends with `end`. A rule can contain a `condition`, an `action`, and a `style` block.

You can find more information [here](https://github.com/Julian-Alberts/i3_notify_bar/blob/master/config.md)
