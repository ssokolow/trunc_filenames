# trunc_filenames

    ssokolow@monolith ~ % trunc_filenames --help
    trunc_filenames 0.1.0
    Rename files and directories to fit length limits.

    WARNING: Will not preserve secondary extensions like .tar.gz

    USAGE:
        trunc_filenames [OPTIONS] [PATH]...

    ARGS:
        <PATH>...    Paths to rename (recursively, if directories)

    OPTIONS:
        -h, --help                 Print help information
            --max-len <MAX_LEN>    Length to truncate to. (Default chosen for rclone name encryption)
                                  [default: 140]
        -n, --dry-run              Don't actually rename files. Just print
        -V, --version              Print version information

## Current shortcomings

1. Requires a POSIX platform because I didn't want to do something hacky and
   then forget, so I only bothered to implement "the length we care about is the
   _encoded_ length" based on `std::os::unix::ffi::OsStrExt`.
2. Doesn't preserve secondary extensions like the `.tar` in `.tar.gz`
3. If the file/directory name already contains bytes that aren't valid UTF-8, it
   won't bother to ensure that the truncation falls on the boundary between
   valid UTF-8 code points.

All of these are because it was a quick itch-scratch I would normally write in
Python and not even revision-control or upload, it works well enough for the
problem it was meant to solve, and solving any of those would have a
significantly lower return on investment.
