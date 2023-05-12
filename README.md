Detachable 3270
===============

Have you ever wanted a TN3270 session that you could use from more
than one place at once?  Have you wanted to be able to close your x3270
window and not lose your session?  This may be for you.

Be warned: this is an extremely overwrought solution to an extremely
niche problem. Who knows, you might find it useful, though...

Documentation
=============

It's all in my head. I'll probably write it down someday...

To get started, though, there are two parts: the server and the client

The Server (d3270d)
-------------------

d3270d wraps b3270 (version 4.2 or greater) and provides access to its
output stream over the network. It takes most of the same arguments as
b3270 (and the b3270 arguments that it *doesn't* take, it silently
ignores).

It does take some additional arguments though:

`-tcp-listen ip:port`: Expose b3270 as a bare TCP stream. Used by d3270console.

`-http-listen ip:port`: Expose a web server with a javascript client, and b3270 available via a websocket at `/api/ws`.

`-connect host[:port]`: Give a machine to connect to at startup. Allows any connect string allowed by b3270.

You should probably give at least one of `tcp-listen` or
`http-listen`. It won't complain if you don't, but neither will it do
anything useful.

Note that, in the above list, `ip` *really* needs to be an
IP address. `localhost` doesn't work. Use `[::]` for all addresses, or `[::1]`
for localhost.

The client (d3270console)
-------------------------

This allows you to connect to a d3270d server from a terminal. It
works on kitty, and you may be lucky elsewhere.

Usage: `d3270console ip:port`

Use the port that you gave to tcp-listen.

Ctrl-C to exit, otherwise the keybindings are the same as the web console.

The client (js3270)
-------------------

Visit the host:port that you gave `-http-listen` in a browser. Enjoy.

Key bindings:

`Tab`: next field
`Shift-Tab`: previous field
`Alt-r`: reset key
`Alt-a`: attn key
`Alt-c`: reconnect b3270 to the remote machine
`Insert`: Toggle insert mode.
`Page up/page down`: page through the scrollback.
`F1 - F12`: PF1-PF12
`Shift-F1 - Shift-F12`: PF13-PF24

If you want anything different, hack the source (search
js3270/src/main.ts for `keymap`, or d3270console for `KeyCode`.)

Building
========

Make sure that you have b3270 (>=4.2) and a recent version of both rust and node.js installed.

In the source directory, run:

```
(cd js3270 && npm install && npm run build)
cargo build --release
```

You'll find the binaries in target/release. The console is embedded in the d3270d binary.

Security
========

Hey look, a squirrel!

No seriously, this is hideously insecure. You can tell b3270 to telnet
to anything on your network, write to files on the local machine, etc,
and nothing in d3270d will stop you. Only run it on a trusted network.

I may add some form of authentication if there's demand, but don't
hold your breath. Similarly, I should probably block some commands
(connect, etc), but that's also not yet implemented.
