These were downloaded and derived from the Open Subtitles data set:
https://opus.nlpl.eu/OpenSubtitles-v2018.php

The specific way in which they were modified has been lost to time, but it's
likely they were just a simple truncation based on target file sizes for
various benchmarks.

The main reason why we have them is that it gives us a way to test similar
inputs on non-ASCII text. Normally this wouldn't matter for a substring search
implementation, but because of the heuristics used to pick a priori determined
"rare bytes" to base a prefilter on, it's possible for this heuristic to do
more poorly on non-ASCII text than one might expect.
