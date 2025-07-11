# moreutils

My from-scratch reimplementation of some of the fantastic utilities from `moreutils`.

Some features are vaguely described, such as the "many common timestamp formats
are supported" for the `ts` program which are not described in the README, man
page, or changelogs. In these situations I glanced over the commit
messages, sans code, for further insights. The rest was trial-and-error and
black-box testing with inputs/outputs against the existing utilities.

## Implemented

- [x] `chronic`
- [x] `combine`
- [x] `errno`
- [x] `ifne`
- [x] `isutf8`
- [x] `mispipe`
- [ ] `parallel`
- [x] `pee`
- [x] `sponge`
- [x] `ts`
- [x] `vidir`
- [x] `vipe`
- [ ] `zrun`

## Will not implement

* `lckdo` - deprecated by `flock`
* `ifdata` - seems to reimplement `ip` or `ifconfig` for script-friendly output
