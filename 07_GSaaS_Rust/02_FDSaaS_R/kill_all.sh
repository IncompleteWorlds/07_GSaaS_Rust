#!/bin/bash

list_files=`ls *pid`
echo $list_files

for f in $list_files; do
    echo $f
    if [ -f $f ]; then
        echo "Killing file: "$f

        pid=`cat $f`
        kill -9 $pid

        rm $f
    fi
done

