# moreutils

My from-scratch reimplementation of some of the fantastic utilities from
[moreutils](http://joeyh.name/code/moreutils/) released under an MIT license.

Some features are vaguely described, such as the "many common timestamp formats
are supported" for the `ts` program which are not described in the README, man
page, or changelogs. In these situations I glanced over the commit
messages, sans code, for further insights. The rest was trial-and-error and
black-box testing with inputs/outputs against the existing utilities. Attempts
were made where possible to support non-UTF8 filenames.

## Implemented

- [x] `chronic`
- [x] `combine`
- [x] `errno`
- [ ] `ifdata`
- [x] `ifne`
- [x] `isutf8`
- [x] `mispipe`
- [x] `parallel`
- [x] `pee`
- [x] `sponge`
- [x] `ts`
- [x] `vidir`
- [x] `vipe`
- [x] `zrun`

## Additional tools added

- `pause` - block forever

## Will not implement

* `lckdo` - deprecated by `flock`
