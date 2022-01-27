#!/bin/sh
clipboards="$(curl -X GET http://localhost:8000/api/get_clipboards)"

KEYS=($((echo $clipboards | jq -r 'keys | @sh') | tr -d \'))

OPTIONS=""
for k in ${KEYS[@]}
do
    OPTIONS+="$k \n"    
done

choice=$(echo -e "$OPTIONS" | rofi -dmenu)
echo $clipboards | jq -r ".$choice" | xclip -sel clip
