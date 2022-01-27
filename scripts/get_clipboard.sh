#!/bin/sh
#Assuming you're running the server on your local machine
curl -X GET http://localhost:8000/api/get_clipboard/server_clipboard | xclip -sel clip
