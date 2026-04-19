seq 3 | awk 'BEGIN { srand() } { print "value=" $1 * int(rand() * 10) }'
