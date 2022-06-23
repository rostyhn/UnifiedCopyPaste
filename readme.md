# Unified Copy Paste
Programs that bring all of your clipboards together in one place, easily accessible inside your local network.

# Usage
Comes with two components: a daemon, and a server. The daemon waits for your clipboard's contents to change, and communicates with a server that you host using websockets. It automatically updates the clipboard contents on the server with your clipboard contents, and any changes on the server are reflected immediately on your local computer.

# Compatibility
Currently, the daemon only works with Linux machines running the X11 windowing system.

The web page has been written with touch-screen devices in mind, which was the original use case - I wanted to write a program that would let me send links back and forth between my phone and my computer without having to email myself everytime. 

# TODO
- 🗸 Options for daemon to target a server running on another machine
- 🗸 Add option to have multiple clipboards available on server from multiple machines
- 🗸 Sync server clipboard with own clipboard
- Way to control the daemon for when content should be pulled / pushed
- GUI for daemon
- Windows support
- Mac support (don't count on it)
- Proper authentication / password protection
