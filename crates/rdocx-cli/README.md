# rdocx-cli

`rdocx-cli` is the command-line interface for inspecting, extracting,
converting, comparing, replacing, validating, and rendering DOCX files. Use
the [`rdocx`](https://docs.rs/rdocx) library when these operations need to run
inside a Rust application.

```sh
cargo install rdocx-cli --version '^0.5'

rdocx inspect report.docx
rdocx text report.docx
rdocx convert report.docx --to pdf -o report.pdf
rdocx validate report.docx
rdocx render report.docx --page 0 -o rendered
```

Run `rdocx --help` or `rdocx <command> --help` for the complete option set.
