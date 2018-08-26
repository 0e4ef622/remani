remani [![Build status](https://travis-ci.org/0e4ef622/remani.svg?branch=master)](https://travis-ci.org/0e4ef622/remani)
==============================================

NOTE: This is still under ~~heavy~~ development. I might publish this on cargo
if it matures to a satisfactory state.

Remake of o2jam/osumania written in Rust.

Basically a modular fully customizable 7k VSRG that aims to support multiple
skin formats (for gameplay at least) and multiple chart formats.

The goal is not to _emulate_ other games, it's just to mimick the interface so
that you don't have to learn how to read a new skin.

Currently requires nightly rust for the `literal` macro matcher feature.

This project began as a semester long coding project for a high school class
where we set our own deadlines and tried to meet them. The aforementioned
deadlines were hosted on GitHub Pages and [can still be seen](https://0e4ef622.github.io/remani/).

Libraries used
==============
* [Piston](https://github.com/PistonDevelopers/piston) [MIT]
* [Image](https://github.com/PistonDevelopers/image) [MIT]
* [CPAL](https://github.com/tomaka/cpal) [Apache 2.0]
* [libmad](https://www.underbit.com/products/mad/) [GPLv2] through [simplemad](https://github.com/bendykst/simple-mad.rs) [MIT]

Other resources used
====================
* [osu!mania default skin](https://osu.ppy.sh/forum/t/129191)
