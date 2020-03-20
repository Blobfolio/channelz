# ChannelZ

ChannelZ is a simple, fast, multi-threaded static Gzip/Brotli encoding tool for the CLI.

Point it toward a single file to generate maximally-compressed Brotli- and Gzip-encoded copies, or point it toward a directory to recursively handle many files en masse.

In directory mode, only files with the following extensions will be looked at:
* css;
* htm(l);
* ico;
* js;
* json;
* mjs;
* svg;
* txt;
* xhtm(l);
* xml;
* xsl

&nbsp;
## Use

```bash
# The help screen:
channelz --help

# Handle one file.
channelz /path/to/file.html

# Handle a whole directory.
channelz /path/to
```

&nbsp;
## Performance

| Method | Time (s) | Difference |
| ---- | ---- | ---- |
| Find/Xargs + Gzip/Brotli | 45.342 | <span style="color: red">+151.2%</span> |
| Find/Parallel + Gzip/Brotli | 23.006 | <span style="color: orange">+27.4%</span> |
| ChannelZ | 18.049 | 🏄 |

Not only does ChannelZ spare you the awful BASH spaghetti, it finishes the job about 2.5× faster.
