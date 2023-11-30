#!/bin/bash

function argument_error {
    echo "$1: expected $2 argument(s), given $3" >&2
    exit 1
}

function check_not_file {
    if [ -f "$1" ]; then
        echo "file already exists: $1" >&2
        exit 1
    fi
}

function check_not_dir {
    if [ -d "$1" ]; then
        echo "directory already exists: $1" >&2
        exit 1
    fi
}

function check_file {
    if [ ! -f "$1" ]; then
        echo "file missing: $1" >&2
        exit 1
    fi
}

function check_dir {
    if [ ! -d "$1" ]; then
        echo "directory missing: $1" >&2
        exit 1
    fi
}

function check_number {
    pattern='^[1-9][0-9]*$'
    if ! [[ $1 =~ $pattern ]]; then
        echo "has to be a positive integer: $1" >&2
        exit 1
    fi
}

function create_year {
    if [[ $# -ne 1 ]]; then
        argument_error "$0" "1" $#
    fi
    name="year_$1"
    base="./src/puzzles"
    mod="$base/mod.rs"
    dir="$base/$name"
    yearmod="$dir/mod.rs"

    echo "create folder for year $1"
    check_not_dir "$dir"
    check_number "$1"
    mkdir -p "$dir"
    echo "mod $name;" >> "$mod"
    touch $yearmod
}

function ensure_year {
    if [[ $# -ne 1 ]]; then
        argument_error "$0" "1" $#
    fi
    base="./src/puzzles"
    name="year_$1"
    dir="$base/$name"

    check_number "$1"
    if [ ! -d "$dir" ]; then
        create_year "$1"
    fi
}

function create_day {
    if [[ $# -ne 2 ]]; then
        argument_error "$0" "2" $#
    fi
    ensure_year "$1"

    template="./template.rs.tt"
    yearname="year_$1"
    dayname="day_$2"
    base="./src/puzzles"
    dir="$base/$yearname"
    mod="$dir/mod.rs"
    basename="$dir/$dayname"
    file="$basename.rs"
    input1="$basename/part_1.txt"
    input2="$basename/part_2.txt"

    echo "create puzzle for day $2"
    check_number "$2"
    check_not_file "$file"
    echo "mod $dayname;" >> "$mod"
    export year=$1
    export day=$2
    envsubst < "$template" > "$file"
    mkdir "$basename"
    touch "$input1"
    touch "$input2"
}

command=$1
shift
if [ "$command" = "create" ]; then
    create_day "$@"
else
    echo "unknown command $1" >&2
    exit 1
fi