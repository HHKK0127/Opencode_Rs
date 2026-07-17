param([string]$path)
$text = [System.IO.File]::ReadAllText($path)
$lines = $text -split "`n"
$total = 0
$depthByLine = @()
for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    $delta = 0
    foreach ($c in $line.ToCharArray()) {
        if ($c -eq '{') { $delta++ }
        elseif ($c -eq '}') { $delta-- }
    }
    $total += $delta
    $depthByLine += @{line=$i+1; depth=$total}
}
# Show lines where depth changes at the boundary of run_app / impl App
$runAppStart = 1458
$implAppStart = 1980
for ($i = $runAppStart - 1; $i -lt $lines.Count; $i++) {
    $d = $depthByLine[$i]
    # Show when depth enters a new level or returns to 0
    $prevDepth = if ($i -gt 0) { $depthByLine[$i-1].depth } else { 0 }
    if ($d.depth -ne $prevDepth -or $d.depth -eq 0 -or $d.depth -eq 1) {
        Write-Output ("{0}: depth={1} :: {2}" -f $d.line, $d.depth, $lines[$i].TrimEnd())
    }
}
Write-Output ("Final depth: {0}" -f $total)
