# Changelog

All notable changes to this project are documented in this file.

## [0.1.0] - 2026-11-20

### Added

- Add SII container detection and unwrapping ([f5cbf53](https://codeberg.org/viniciusdof/siir/commit/f5cbf53ca603793da7a18803f20485288df48f82))
- Add text document model and edit API ([1224dea](https://codeberg.org/viniciusdof/siir/commit/1224deaef0764a30cd0c5ee5b99542d0d467413d))
- Add whole-file open and plain text write ([69fb5b0](https://codeberg.org/viniciusdof/siir/commit/69fb5b0b5208f4b23fd48d94df7a59a4f4b7d512))
- Add BSII binary decoder ([2642c9f](https://codeberg.org/viniciusdof/siir/commit/2642c9fb200ed5c9334107cdd2dea7148d22b3fc))
- Add BSII ordinal arrays and remove version helper ([16b1f54](https://codeberg.org/viniciusdof/siir/commit/16b1f54447a0776573069afb7f64bedd42016d5b))
- Iterate every unit and attribute ([7ceb888](https://codeberg.org/viniciusdof/siir/commit/7ceb888630db499453724e516240a6f097126bfa))
- Add save read model and truck commands ([e0adeac](https://codeberg.org/viniciusdof/siir/commit/e0adeac484adc0da42c7d8c9f1a4dee7c83d9645))
- Add trailer commands ([b79b723](https://codeberg.org/viniciusdof/siir/commit/b79b72304a65b06d94152871705a709b6ada0507))
- Add economy, profile, and config support ([af60201](https://codeberg.org/viniciusdof/siir/commit/af602010a7a448540c93e3ead79dd9507585e8e3))
- Add truck catalog and snapshots ([97e7445](https://codeberg.org/viniciusdof/siir/commit/97e7445cb736f01da2af7198485bdce2308b5bab))
- Add truck part and accessory commands ([2abf017](https://codeberg.org/viniciusdof/siir/commit/2abf017cc8bee2c555a7c4f8a327de1508518d58))
- Add used truck price command ([1826354](https://codeberg.org/viniciusdof/siir/commit/1826354e8af1ce15e004d79a35defdd30f9c9bb0))
- Add color conversion ([5e332e6](https://codeberg.org/viniciusdof/siir/commit/5e332e62f6ef29278c711839ad6ec92b57abad3e))
- Add owned units and value parsing ([df389f8](https://codeberg.org/viniciusdof/siir/commit/df389f85951f66ce15669fa2b81196e8425d9d48))
- Add accessory export and import ([7db718c](https://codeberg.org/viniciusdof/siir/commit/7db718c3a410d3858066f89309f7081c98f4a2a3))
- Add plate country, market, and job commands ([a56c229](https://codeberg.org/viniciusdof/siir/commit/a56c229a922070660eb75bd0375eead69dfee773))
- Add market catalogs and driver commands ([3e6e98c](https://codeberg.org/viniciusdof/siir/commit/3e6e98ce44246f873bd4d4845b42471467593398))
- Add bus and car read model and commands ([4bbd1d0](https://codeberg.org/viniciusdof/siir/commit/4bbd1d0f503d87193eaa956540586de3fb9dbe87))
- Discover profiles under OS, Steam, and Proton ([99d1c17](https://codeberg.org/viniciusdof/siir/commit/99d1c17e162de84a14a216660ced870c7ff2fb6d))
- Own the parsed document in siir::File ([caaf817](https://codeberg.org/viniciusdof/siir/commit/caaf817d63a7b625beac48de775cc43f9259ea74))
- Add a session that saves atomically ([baa441c](https://codeberg.org/viniciusdof/siir/commit/baa441c4dbd69af3b035cab2a7f24cdf227aa4b1))
- Load the catalog from committed snapshots ([4e7374e](https://codeberg.org/viniciusdof/siir/commit/4e7374e3e65d10767defc5437ea7318218a792bb))
- Report discovered profiles with their origin ([35b73e2](https://codeberg.org/viniciusdof/siir/commit/35b73e2d2e2a3d6ed056d2977ac23fc0ccdde73e))
- Expose save game file path ([a9edeac](https://codeberg.org/viniciusdof/siir/commit/a9edeaca511865adaca7537316fad7b7dc239895))
- Add the read-only desktop GUI ([cb569d6](https://codeberg.org/viniciusdof/siir/commit/cb569d6eda2970fbf6db20e8898f072237ab492d))
- Add the economy editor, game terms, and pt-BR ([cf627f6](https://codeberg.org/viniciusdof/siir/commit/cf627f604548e515b2194a25a3ab71ba05796524))
- Persist custom folders, the last game, and the locale ([7a6d8d7](https://codeberg.org/viniciusdof/siir/commit/7a6d8d77ef3a1ab510321fcfdfdb2ff63b69aece))
- Add visited cities and dealerships to the economy editor ([5911671](https://codeberg.org/viniciusdof/siir/commit/591167125058c94b5fd309a6ed89a9e02601d8b0))
- Add profile name editing to the GUI ([ab778d7](https://codeberg.org/viniciusdof/siir/commit/ab778d79c6c3485a75e304f39df5669d388f8019))
- Confirm unsaved changes in a modal dialog ([d255eb8](https://codeberg.org/viniciusdof/siir/commit/d255eb8e40e4e60dcb2bf311f15e56f5952ea7d8))
- Group experience and counts in the game's number format ([eaa19a7](https://codeberg.org/viniciusdof/siir/commit/eaa19a7da2e848eed52005dcf329fed990713732))
- Finish the M2 editor with overview links, the OS language, and TOML settings ([8b0fd5b](https://codeberg.org/viniciusdof/siir/commit/8b0fd5b4786e44a4c6636e3bfb3198f3c740d213))
- Add the vehicles and trailers screens ([b2e6bb1](https://codeberg.org/viniciusdof/siir/commit/b2e6bb14b481a9019e5c775e4a1695de10ecafba))
- Drive levels, money, and plate country from the game's data ([98b4706](https://codeberg.org/viniciusdof/siir/commit/98b47063543d84b04591e57975cc1bfeac108c06))
- Add catalog part pickers and name trailers, cities, and parts from the game's locale ([b51da35](https://codeberg.org/viniciusdof/siir/commit/b51da3512ef9e3ebe80443948877a9f9a0a6df7d))

### Fixed

- Bound ScsC decompression and report payload offsets ([decaace](https://codeberg.org/viniciusdof/siir/commit/decaace4822e5207ae59fe81c8faf4e102d4c2fa))
- Correct save edits, profile archives, and binary decoding ([f2b0604](https://codeberg.org/viniciusdof/siir/commit/f2b0604020458fc049c2423a73d5314eff44fdc2))
- Decode int pairs and namespaced classes ([0b6826c](https://codeberg.org/viniciusdof/siir/commit/0b6826c3ac59ab94a392156fd6eed458b2fcfe5d))
- Read the save folder inside a profile ([a897699](https://codeberg.org/viniciusdof/siir/commit/a89769996e77a8c5ceb25705d275c617e7931c4e))

### Changed

- Read attribute values as raw text, not a typed kind ([16c1703](https://codeberg.org/viniciusdof/siir/commit/16c1703a0df71472a45745b55c39caf9e13fefd8))
- Move the library into a workspace ([683b964](https://codeberg.org/viniciusdof/siir/commit/683b9647f5877ac9d70992c997f24023693c8a36))
- Rename truck types to vehicle and repair all wear attributes ([9574524](https://codeberg.org/viniciusdof/siir/commit/9574524fbdb82bb921751cec25893e1dc13b552f))

### Documentation

- Add the readme and the MIT licence ([bb17389](https://codeberg.org/viniciusdof/siir/commit/bb173897ed0dc43c3a7bef9a4fbf620896c4dcb9))
- Remove outdated app feature summary from README ([f824c64](https://codeberg.org/viniciusdof/siir/commit/f824c6415b98540c7d392af32fb079eb1ee5f9f4))
