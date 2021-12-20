# Unified Copy Paste
Programs that bring all of your clipboards together in one place, easily accessible inside your local network.

# Usage
Comes with two components: a daemon, and a server. The daemon waits for your clipboard's contents to change, and then uses a `POST` request to get them on the server component. You can then login to the server to see your clipboard's contents from another machine, and copy / modify it.

While there isn't a seperate executable to copy the server's clipboard contents yet, included within `scripts` is a sample .sh script that you could bind to a key in your favorite window / desktop manager to copy the server's clipboard contents.

## Sample script 
The script itself is a simple one-liner:
```
curl -X GET http://localhost:8000/api/get_clipboard | xclip -sel clip
```

This of course assumes the server is running on the same machine you're calling it from. You can easily change it to whatever IP the server clipboard is available at.

I have this script bound to my i3 config like so:

```
bindsym $mod+Shift+c exec "PATH_TO_PROGRAM/scripts/get_clipboard.sh"
```
# Compatability
Currently, the daemon only works with Linux machines running the X11 windowing system.

# TODO
- Options for daemon to target a server running on another machine
- Add option to have multiple clipboards available on server from multiple machines
- Windows support
- Mac support (don't count on it)
- Authentication / password protection
- Some strings cannot be copied, figure that out
- "Quiet mode" for daemon to stop sending clipboard output for some time
