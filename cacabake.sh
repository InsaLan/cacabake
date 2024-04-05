#! /bin/bash

sleep_fraction() {
  /usr/bin/perl -e "select(undef, undef, undef, $1)"
}

videosext=("mp4 mpv mkv mov avi")
input=0 # input is wrong by default

if ! [ -f "$1" ]; then
    echo "File not found."
    exit 1
fi

if [[ ${1##*.} = baked ]]; then
    input=1 # baked file to show
else
    for ext in $videosext
    do
        if [[ ${1##*.} = $ext ]]; then
            input=2 # video to bake
        fi
    done
fi

if [ $input -eq 0 ]; then
    echo "Wrong file format. Use a video or baked file."
elif [ $input -eq 1 ]; then # playback
    framerate=$(sed -n '1p' $1)
#    for i in ($1)
#    do
        cat "${p}"
        sleep_fraction 0.01428
    done
elif [ $input -eq 2 ]; then # bake
    touch ${1%%.*}.baked
    ffprobe $1 2>&1| grep ",* fps" | cut -d "," -f 5 | cut -d " " -f 2 > ${1%%.*}.baked 
    mpv $1 --vo=caca --untimed --framedrop=no --no-config --no-terminal >> ${1%%.*}.baked
    echo "Bake successful"
fi
