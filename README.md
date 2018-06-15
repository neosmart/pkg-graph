# `pkg-graph`
_A graph visualizer for FreeBSD's `pkg` package management utility._

## About
`pkg-graph` is a command-line utility to generate a dependency graph in DOT
syntax (for use with graphviz and others) to represent the dependency tree of
installed packages. It can be used to visually analyze the package dependency
chain and understand why certain packages are pulled in.

## Use
`pkg-graph` can be compiled with rust's `cargo` as follows:

```bash
cargo build --release
cargo install
```

After installation, it is available as `pkg-graph` from the command line.

### Generate a complete dependency graph
```bash
pkg-graph
```

The above will generate a complete dependency graph of all packages installed on
the system, including zero-dependency packages and leaf nodes.

### Generate a selective dependency graph
```bash
pkg-graph PKG1 [PKG2 [PKG3 ..]]
```

When executed as in the sample above, `pkg-graph` will restrict its output to
the minimum dependencies required to satisfy the packages named at the command
line.

## Sample Output

```fish
~/pkg-graph> uname -r
12.0-CURRENT
~/pkg-graph> pkg-graph git
digraph {
        "pcre" -> "git";
        "ca_root_nss" -> "curl";
        "curl" -> "git";
        "libffi" -> "python27";
        "perl5" -> { "p5-HTML-Parser" "git" "p5-Digest-HMAC" "p5-CGI" "p5-Error" "p5-Authen-SASL" "p5-HTML-Tagset" "p5-GSSAPI" };
        "cvsps" -> "git";
        "p5-Error" -> "git";
        "indexinfo" -> { "gettext-runtime" "readline" "libffi" };
        "gettext-runtime" -> { "git" "python27" };
        "p5-Digest-HMAC" -> "p5-Authen-SASL";
        "expat" -> "git";
        "p5-CGI" -> "git";
        "p5-HTML-Tagset" -> "p5-HTML-Parser";
        "readline" -> "python27";
        "python27" -> "git";
        "p5-GSSAPI" -> "p5-Authen-SASL";
        "git";
        "libnghttp2" -> "curl";
        "p5-HTML-Parser" -> "p5-CGI";
        "p5-Authen-SASL" -> "git";
}
```

Which results [in the following output](https://goo.gl/hozu1b) when charted
with a graphviz-compatible visualization tool.

## License and Copyright

`pkg-graph` is developed by Mahmoud Al-Qudsi of [NeoSmart
Technologies](https://neosmart.net/) and is released to the general public
under the terms of the two-clause simplified BSD license. Refer to the
`LICENSE` file for more information.

