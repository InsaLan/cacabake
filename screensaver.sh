#! /bin/bash

tty=$(tty)

idle_player() {
	while true;
	do
		echo "foo"
		if (( $(echo "$(w | grep "$(whoami)" | awk '{print $5}' | sed 's/s//') > $1" |bc -l))); then
			echo "bar"
			./target/release/cacabake "$2" -l -a &
			pid=$!
			while read -r -t 0; do read -r; done
			read -n 1 -s -r
			kill $pid
		fi
		sleep 10
	done
}

idle_player "$1" "$2" < "$tty" &