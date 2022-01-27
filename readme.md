# Unified Copy Paste
Programs that bring all of your clipboards together in one place, easily accessible inside your local network.

# Usage
Comes with two components: a daemon, and a server. The daemon waits for your clipboard's contents to change, and then uses a `POST` request to get them on the server component. You can then login to the server to see your clipboard's contents from another machine, and copy / modify it.

While there isn't a separate executable to copy the server's clipboard contents yet, included within `scripts` is a sample .sh script that you could bind to a key in your favorite window / desktop manager to copy the server's clipboard contents.

## Sample script 
The script `get_clipboard.sh` itself is a simple one-liner:
```
curl -X GET http://localhost:8000/api/get_clipboard | xclip -sel clip
```

This of course assumes the server is running on the same machine you're calling it from. You can easily change it to whatever IP the server clipboard is available at.

I have this script bound to my i3 config like so:

```
bindsym $mod+Shift+c exec "PATH_TO_DAEMON/scripts/get_clipboard.sh"
```

I have since added a rofi script that shows a list of the clipboards available on the server, and copies their contents on selection, which is pretty cool. Theoretically, should work with dmenu as well - I'll make the script agnostic later.

# Compatibility
Currently, the daemon only works with Linux machines running the X11 windowing system.

The web page has been written with touch-screen devices in mind, which was the original use case - I wanted to write a program that would let me send links back and forth between my phone and my computer without having to email myself everytime. 

# Testing
Included inside the tests folder is a file called api_tests.org that lets you query the server using emacs verb. In the future, I'll add the curl equivalents underneath, but it should be fairly obvious what the queries should be.

# TODO
- 🗸 Options for daemon to target a server running on another machine
- 🗸 Add option to have multiple clipboards available on server from multiple machines
- 🗸 Added sample rofi menu script
- Rewrite some portions of the code more rustily
- Beautify web page
- Windows support
- Mac support (don't count on it)
- Authentication / password protection
- Some strings cannot be copied, figure that out
