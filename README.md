# what_error

ever wanted to look up that weird windows error code?

this can do that.
 
supports NT and DOS error codes

```
what_error/0.1.0

usage:
    -e, --error   <error_code>
        the eerror code to look up.
        this can be either as an integer or a hex literal
        e.g. -2147467261 or 0x80004003

    --nt
        assume its an NT status code.
        some error codes are shared (e.g. '5') between DOS and NT.
        this forces it to look it up as an NT status code

    -h, --help
        writes this help message

    -v, --version
        writes the current version
```
