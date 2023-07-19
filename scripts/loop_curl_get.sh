#! /bin/bash

# This script executes a curl GET request to the specified URL
# in a loop for the specified number of times.

# Usage: ./loop_curl_get.sh <number of times>
# Example: ./loop_curl_get.sh 10

# Check if the number of arguments is correct
if [ $# -ne 1 ]; then
    echo "Usage: ./loop_curl_get.sh <number of times>"
    exit 1
fi

# Check if the argument is a number
if ! [[ $1 =~ ^[0-9]+$ ]]; then
    echo "The argument must be a number"
    exit 1
fi

# Check if the argument is greater than 0
if [ "$1" -le 0 ]; then
    echo "The argument must be greater than 0"
    exit 1
fi

start=$(date +%s)

# Execute the curl GET request in a loop
for (( i=1; i<=$1; i++ ))
do
    curl -s -X GET http://localhost:9999/ >> /dev/null
done

end=$(date +%s)
elapsed=$((end-start))
echo "Elapsed time: $elapsed seconds"
