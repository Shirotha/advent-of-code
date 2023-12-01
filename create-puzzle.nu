#!~/.cargo/bin/nu

def main [year: int, day: int] {
    let mRoot = "mod.rs"
    let dYear = $"year($year)"
    let mYear = $"($dYear)/mod.rs"
    let dDay = $"($dYear)/day($day)"
    
    print $"(ansi green)create puzzle(ansi reset)"
    let dRoot = "./src/puzzles"
    if ($"($dRoot)/($dDay).rs" | path exists) {
        print $"(ansi red_bold)!(ansi reset) (ansi red)already exists(ansi reset)"
        exit 1
    }

    if ($"($dRoot)/($dYear)" | path type) != "dir" {
        print "  register year"
        $"pub mod year($year);\n" | save --append $"($dRoot)/($mRoot)"
        mkdir $"($dRoot)/($dYear)"
        touch $"($dRoot)/($mYear)"
    }
    print "  register day"
    $"pub mod day($day);\n" | save --append $"($dRoot)/($mYear)"

    print "  apply template"
    mkdir $"($dRoot)/($dDay)"
    open "./puzzle.rs.tt" --raw
        | str replace -a "$day" $"($day)"
        | str replace -a "$year" $"($year)"
        | save $"($dRoot)/($dDay).rs"
    touch $"($dRoot)/($dDay)/part1.txt"
    touch $"($dRoot)/($dDay)/part2.txt"

    print $"(ansi green)create bench(ansi reset)"
    let dRoot = "./benches/puzzles"
    
    if ($"($dRoot)/($dYear)" | path type) != "dir" {
        print "  register year"
        $"mod year($year);\n" | save --append $"($dRoot)/($mRoot)"
        mkdir $"($dRoot)/($dYear)"
        touch $"($dRoot)/($mYear)"
    }
    print "  register day"
    $"mod day($day);\n" | save --append $"($dRoot)/($mYear)"

    print "  apply template"
    mkdir $"($dRoot)/($dYear)"
    open "./bench.rs.tt" --raw
        | str replace -a "$day" $"($day)"
        | str replace -a "$year" $"($year)"
        | save $"($dRoot)/($dDay).rs"
}
