# Third-party notices

MCC GUI V2 is released under the **MIT License** — see [LICENSE](LICENSE). It builds on the
third-party components listed here; this file records them and the license each one is under.

Generated on **2026-10-16** for version **2.0.0**, from the versions locked in
`package-lock.json` and `src-tauri/Cargo.lock` — the same inputs the Windows installer was
built from.

## Scope

- **JavaScript** (40 packages): the **production** dependency tree of
  `package.json`, including everything those packages pull in. Build tools (Vite, ESLint,
  Tailwind, Vitest) are not part of it, because they never end up in the app.
- **Rust** (474 crates): everything resolved from `src-tauri/Cargo.lock`.
  That resolution is platform-wide, so a handful of crates that only build on other
  platforms are listed too, even though they are not compiled into the Windows binary.

## How to regenerate this file

```bash
# npm, production tree only (license-checker-rseidelsohn 5.0.1)
npx --yes license-checker-rseidelsohn@5.0.1 --production --json --start . > npm.json

# Rust crates, together with the license texts (cargo-bundle-licenses 4.2.0)
cd src-tauri && cargo bundle-licenses --format yaml --output THIRDPARTY.yml && cd ..

# and the file itself
node scripts/build-third-party-notices.mjs npm.json src-tauri/THIRDPARTY.yml
```

## Licenses in use

| License | npm packages | Rust crates |
|---|---|---|
| MIT | 35 | 437 |
| Apache-2.0 | 2 | 309 |
| Zlib | — | 24 |
| Unicode-3.0 | — | 19 |
| BSD-3-Clause | 1 | 8 |
| Unlicense | — | 7 |
| Apache-2.0 WITH LLVM-exception | — | 5 |
| MPL-2.0 | — | 5 |
| BSD-2-Clause | 1 | 2 |
| ISC | 2 | 1 |
| BSL-1.0 | — | 2 |
| LGPL-2.1-or-later | — | 2 |
| 0BSD | — | 1 |
| CC0-1.0 | — | 1 |
| MIT-0 | — | 1 |

No component here is under a strong-copyleft license (GPL or AGPL), and none of them is
modified: the app only links against them as published. The licenses worth naming explicitly:

- **MPL-2.0** — `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext` and `selectors`.
  MPL-2.0 is file-level copyleft: it covers those crates' own source files and explicitly
  allows them to be combined with code under other licenses, which is what happens here. The
  crates are used unmodified and their sources stay available upstream (Repository column).
- **LGPL-2.1-or-later** — appears only in `r-efi`, which is licensed
  `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; this project takes the MIT/Apache-2.0 option.
- **Unlicense**, **CC0-1.0**, **MIT-0** and **0BSD** are permissive to the point of asking for
  no attribution; they are listed here for completeness. **BSL-1.0** (Boost) is permissive as
  well, but asks for its notice to be kept with copies — the components under it are in the
  table below, with links to their upstream sources.

**Minecraft Console Client is not on this list.** It is not a dependency of this project and
is never bundled or redistributed with it — the app only starts the `MinecraftClient.exe` you
point it at, as a separate process. See the licence section of the README.

## JavaScript packages

| Component | Version | License | Repository |
|---|---|---|---|
| `@babel/helper-string-parser` | 7.29.7 | MIT | [github.com/babel/babel](https://github.com/babel/babel) |
| `@babel/helper-validator-identifier` | 7.29.7 | MIT | [github.com/babel/babel](https://github.com/babel/babel) |
| `@babel/parser` | 7.29.9 | MIT | [github.com/babel/babel](https://github.com/babel/babel) |
| `@babel/types` | 7.29.8 | MIT | [github.com/babel/babel](https://github.com/babel/babel) |
| `@intlify/core-base` | 11.4.13 | MIT | [github.com/intlify/vue-i18n](https://github.com/intlify/vue-i18n) |
| `@intlify/devtools-types` | 11.4.13 | MIT | [github.com/intlify/vue-i18n](https://github.com/intlify/vue-i18n) |
| `@intlify/message-compiler` | 11.4.13 | MIT | [github.com/intlify/vue-i18n](https://github.com/intlify/vue-i18n) |
| `@intlify/shared` | 11.4.13 | MIT | [github.com/intlify/vue-i18n](https://github.com/intlify/vue-i18n) |
| `@jridgewell/sourcemap-codec` | 1.6.0 | MIT | [github.com/jridgewell/sourcemaps](https://github.com/jridgewell/sourcemaps) |
| `@lucide/vue` | 1.50.0 | ISC | [github.com/lucide-icons/lucide](https://github.com/lucide-icons/lucide) |
| `@tauri-apps/api` | 2.12.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `@vue/compiler-core` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/compiler-dom` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/compiler-sfc` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/compiler-ssr` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/devtools-api` | 6.6.4 | MIT | [github.com/vuejs/vue-devtools](https://github.com/vuejs/vue-devtools) |
| `@vue/devtools-api` | 8.2.1 | MIT | [github.com/vuejs/devtools](https://github.com/vuejs/devtools) |
| `@vue/devtools-kit` | 8.2.1 | MIT | [github.com/vuejs/devtools](https://github.com/vuejs/devtools) |
| `@vue/devtools-shared` | 8.2.1 | MIT | [github.com/vuejs/devtools](https://github.com/vuejs/devtools) |
| `@vue/reactivity` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/runtime-core` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/runtime-dom` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/server-renderer` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `@vue/shared` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `birpc` | 2.9.0 | MIT | [github.com/antfu-collective/birpc](https://github.com/antfu-collective/birpc) |
| `csstype` | 3.2.3 | MIT | [github.com/frenic/csstype](https://github.com/frenic/csstype) |
| `entities` | 7.0.1 | BSD-2-Clause | [github.com/fb55/entities](https://github.com/fb55/entities) |
| `estree-walker` | 2.0.2 | MIT | [github.com/Rich-Harris/estree-walker](https://github.com/Rich-Harris/estree-walker) |
| `hookable` | 5.5.3 | MIT | [github.com/unjs/hookable](https://github.com/unjs/hookable) |
| `magic-string` | 0.30.21 | MIT | [github.com/Rich-Harris/magic-string](https://github.com/Rich-Harris/magic-string) |
| `nanoid` | 3.3.19 | MIT | [github.com/ai/nanoid](https://github.com/ai/nanoid) |
| `nostics` | 1.3.0 | MIT | [github.com/vercel-labs/nostics](https://github.com/vercel-labs/nostics) |
| `perfect-debounce` | 2.1.0 | MIT | [github.com/unjs/perfect-debounce](https://github.com/unjs/perfect-debounce) |
| `picocolors` | 1.1.1 | ISC | [github.com/alexeyraspopov/picocolors](https://github.com/alexeyraspopov/picocolors) |
| `pinia` | 4.0.3 | MIT | [github.com/vuejs/pinia](https://github.com/vuejs/pinia) |
| `postcss` | 8.5.28 | MIT | [github.com/postcss/postcss](https://github.com/postcss/postcss) |
| `source-map-js` | 1.2.2 | BSD-3-Clause | [github.com/7rulnik/source-map-js](https://github.com/7rulnik/source-map-js) |
| `typescript` | 6.0.3 | Apache-2.0 | [github.com/microsoft/TypeScript](https://github.com/microsoft/TypeScript) |
| `vue` | 3.5.43 | MIT | [github.com/vuejs/core](https://github.com/vuejs/core) |
| `vue-i18n` | 11.4.13 | MIT | [github.com/intlify/vue-i18n](https://github.com/intlify/vue-i18n) |

## Rust crates

| Component | Version | License | Repository |
|---|---|---|---|
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | [github.com/oyvindln/adler2](https://github.com/oyvindln/adler2) |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT | [github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| `alloc-no-stdlib` | 3.0.0 | BSD-3-Clause | [github.com/dropbox/rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) |
| `alloc-stdlib` | 0.3.0 | BSD-3-Clause | [github.com/dropbox/rust-alloc-no-stdlib](https://github.com/dropbox/rust-alloc-no-stdlib) |
| `android_system_properties` | 0.1.6 | MIT OR Apache-2.0 | [github.com/nical/android_system_properties](https://github.com/nical/android_system_properties) |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | [github.com/dtolnay/anyhow](https://github.com/dtolnay/anyhow) |
| `arboard` | 3.6.1 | MIT OR Apache-2.0 | [github.com/1Password/arboard](https://github.com/1Password/arboard) |
| `async-broadcast` | 0.7.2 | MIT OR Apache-2.0 | [github.com/smol-rs/async-broadcast](https://github.com/smol-rs/async-broadcast) |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT | [github.com/smol-rs/async-channel](https://github.com/smol-rs/async-channel) |
| `async-executor` | 1.14.0 | Apache-2.0 OR MIT | [github.com/smol-rs/async-executor](https://github.com/smol-rs/async-executor) |
| `async-io` | 2.6.0 | Apache-2.0 OR MIT | [github.com/smol-rs/async-io](https://github.com/smol-rs/async-io) |
| `async-lock` | 3.4.2 | Apache-2.0 OR MIT | [github.com/smol-rs/async-lock](https://github.com/smol-rs/async-lock) |
| `async-process` | 2.5.0 | Apache-2.0 OR MIT | [github.com/smol-rs/async-process](https://github.com/smol-rs/async-process) |
| `async-recursion` | 1.2.0 | MIT OR Apache-2.0 | [github.com/dcchut/async-recursion](https://github.com/dcchut/async-recursion) |
| `async-signal` | 0.2.14 | Apache-2.0 OR MIT | [github.com/smol-rs/async-signal](https://github.com/smol-rs/async-signal) |
| `async-task` | 4.7.1 | Apache-2.0 OR MIT | [github.com/smol-rs/async-task](https://github.com/smol-rs/async-task) |
| `async-trait` | 0.1.92 | MIT OR Apache-2.0 | [github.com/dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| `atk` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `atk-sys` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | [github.com/smol-rs/atomic-waker](https://github.com/smol-rs/atomic-waker) |
| `base64` | 0.21.7 | MIT OR Apache-2.0 | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `base64` | 0.22.1 | MIT OR Apache-2.0 | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `base64` | 0.23.1 | MIT OR Apache-2.0 | [github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `bit-set` | 0.8.0 | Apache-2.0 OR MIT | [github.com/contain-rs/bit-set](https://github.com/contain-rs/bit-set) |
| `bit-vec` | 0.8.0 | Apache-2.0 OR MIT | [github.com/contain-rs/bit-vec](https://github.com/contain-rs/bit-vec) |
| `bitflags` | 1.3.2 | MIT/Apache-2.0 | [github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | [github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 | [github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `block2` | 0.6.2 | MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `blocking` | 1.7.0 | Apache-2.0 OR MIT | [github.com/smol-rs/blocking](https://github.com/smol-rs/blocking) |
| `brotli` | 9.0.0 | BSD-3-Clause AND MIT | [github.com/dropbox/rust-brotli](https://github.com/dropbox/rust-brotli) |
| `brotli-decompressor` | 6.0.1 | BSD-3-Clause/MIT | [github.com/dropbox/rust-brotli-decompressor](https://github.com/dropbox/rust-brotli-decompressor) |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 | [github.com/fitzgen/bumpalo](https://github.com/fitzgen/bumpalo) |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | [github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| `byteorder` | 1.5.0 | Unlicense OR MIT | [github.com/BurntSushi/byteorder](https://github.com/BurntSushi/byteorder) |
| `byteorder-lite` | 0.1.0 | Unlicense OR MIT | [github.com/image-rs/byteorder-lite](https://github.com/image-rs/byteorder-lite) |
| `bytes` | 1.12.1 | MIT | [github.com/tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| `cairo-rs` | 0.18.5 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `cairo-sys-rs` | 0.18.2 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `camino` | 1.2.6 | MIT OR Apache-2.0 | [github.com/camino-rs/camino](https://github.com/camino-rs/camino) |
| `cargo_metadata` | 0.19.2 | MIT | [github.com/oli-obk/cargo_metadata](https://github.com/oli-obk/cargo_metadata) |
| `cargo-platform` | 0.1.9 | MIT OR Apache-2.0 | [github.com/rust-lang/cargo](https://github.com/rust-lang/cargo) |
| `cesu8` | 1.1.0 | Apache-2.0/MIT | [github.com/emk/cesu8-rs](https://github.com/emk/cesu8-rs) |
| `cfb` | 0.14.0 | MIT | [github.com/mdsteele/rust-cfb](https://github.com/mdsteele/rust-cfb) |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | [github.com/rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 | [github.com/chronotope/chrono](https://github.com/chronotope/chrono) |
| `clipboard-win` | 5.4.1 | BSL-1.0 | [github.com/DoumanAsh/clipboard-win](https://github.com/DoumanAsh/clipboard-win) |
| `combine` | 4.6.8 | MIT | [github.com/Marwes/combine](https://github.com/Marwes/combine) |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT | [github.com/smol-rs/concurrent-queue](https://github.com/smol-rs/concurrent-queue) |
| `cookie` | 0.18.2 | MIT OR Apache-2.0 | [github.com/SergioBenitez/cookie-rs](https://github.com/SergioBenitez/cookie-rs) |
| `core-foundation` | 0.10.1 | MIT OR Apache-2.0 | [github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | [github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-graphics` | 0.25.0 | MIT OR Apache-2.0 | [github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-graphics-types` | 0.2.0 | MIT OR Apache-2.0 | [github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | [github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `crc32fast` | 1.5.2 | MIT OR Apache-2.0 | [github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| `crossbeam-channel` | 0.5.17 | MIT OR Apache-2.0 | [github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | [github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| `crunchy` | 0.2.4 | MIT | [github.com/eira-fransham/crunchy](https://github.com/eira-fransham/crunchy) |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 | [github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| `cssparser` | 0.37.0 | MPL-2.0 | [github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| `cssparser-macros` | 0.7.1 | MPL-2.0 | [github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| `ctor` | 1.0.13 | Apache-2.0 OR MIT | [github.com/mmastrac/linktime](https://github.com/mmastrac/linktime) |
| `darling` | 0.21.3 | MIT | [github.com/TedDriggs/darling](https://github.com/TedDriggs/darling) |
| `darling_core` | 0.21.3 | MIT | [github.com/TedDriggs/darling](https://github.com/TedDriggs/darling) |
| `darling_macro` | 0.21.3 | MIT | [github.com/TedDriggs/darling](https://github.com/TedDriggs/darling) |
| `dbus` | 0.9.12 | Apache-2.0/MIT | [github.com/diwic/dbus-rs](https://github.com/diwic/dbus-rs) |
| `deranged` | 0.5.8 | MIT OR Apache-2.0 | [github.com/jhpratt/deranged](https://github.com/jhpratt/deranged) |
| `derive_more` | 2.1.1 | MIT | [github.com/JelteF/derive_more](https://github.com/JelteF/derive_more) |
| `derive_more-impl` | 2.1.1 | MIT | [github.com/JelteF/derive_more](https://github.com/JelteF/derive_more) |
| `digest` | 0.10.7 | MIT OR Apache-2.0 | [github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| `dirs` | 7.0.0 | MIT OR Apache-2.0 | [codeberg.org/dirs/dirs-rs](https://codeberg.org/dirs/dirs-rs) |
| `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 | [github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | [github.com/yaahc/displaydoc](https://github.com/yaahc/displaydoc) |
| `dlopen2` | 0.8.2 | MIT | [github.com/OpenByteDev/dlopen2](https://github.com/OpenByteDev/dlopen2) |
| `dlopen2_derive` | 0.4.3 | MIT | [github.com/OpenByteDev/dlopen2](https://github.com/OpenByteDev/dlopen2) |
| `dom_query` | 0.28.0 | MIT | [github.com/niklak/dom_query](https://github.com/niklak/dom_query) |
| `downcast-rs` | 1.2.1 | MIT/Apache-2.0 | [github.com/marcianx/downcast-rs](https://github.com/marcianx/downcast-rs) |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | [github.com/rust-windowing/winit](https://github.com/rust-windowing/winit) |
| `dtoa` | 1.0.11 | MIT OR Apache-2.0 | [github.com/dtolnay/dtoa](https://github.com/dtolnay/dtoa) |
| `dtoa-short` | 0.3.5 | MPL-2.0 | [github.com/upsuper/dtoa-short](https://github.com/upsuper/dtoa-short) |
| `dunce` | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | [gitlab.com/kornelski/dunce](https://gitlab.com/kornelski/dunce) |
| `dyn-clone` | 1.0.20 | MIT OR Apache-2.0 | [github.com/dtolnay/dyn-clone](https://github.com/dtolnay/dyn-clone) |
| `embed_plist` | 1.2.2 | MIT OR Apache-2.0 | [github.com/nvzqz/embed-plist-rs](https://github.com/nvzqz/embed-plist-rs) |
| `endi` | 1.1.1 | MIT | [github.com/zeenix/endi](https://github.com/zeenix/endi) |
| `enumflags2` | 0.7.12 | MIT OR Apache-2.0 | [github.com/meithecatte/enumflags2](https://github.com/meithecatte/enumflags2) |
| `enumflags2_derive` | 0.7.12 | MIT OR Apache-2.0 | [github.com/meithecatte/enumflags2](https://github.com/meithecatte/enumflags2) |
| `equivalent` | 1.0.2 | Apache-2.0 OR MIT | [github.com/indexmap-rs/equivalent](https://github.com/indexmap-rs/equivalent) |
| `erased-serde` | 0.4.10 | MIT OR Apache-2.0 | [github.com/dtolnay/erased-serde](https://github.com/dtolnay/erased-serde) |
| `errno` | 0.3.14 | MIT OR Apache-2.0 | [github.com/lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| `error-code` | 3.4.0 | BSL-1.0 | [github.com/DoumanAsh/error-code](https://github.com/DoumanAsh/error-code) |
| `event-listener` | 5.4.2 | Apache-2.0 OR MIT | [github.com/smol-rs/event-listener](https://github.com/smol-rs/event-listener) |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT | [github.com/smol-rs/event-listener-strategy](https://github.com/smol-rs/event-listener-strategy) |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | [github.com/smol-rs/fastrand](https://github.com/smol-rs/fastrand) |
| `fax` | 0.2.7 | MIT | [github.com/pdf-rs/fax](https://github.com/pdf-rs/fax) |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | [github.com/image-rs/fdeflate](https://github.com/image-rs/fdeflate) |
| `field-offset` | 0.3.6 | MIT OR Apache-2.0 | [github.com/Diggsey/rust-field-offset](https://github.com/Diggsey/rust-field-offset) |
| `fixedbitset` | 0.5.7 | MIT OR Apache-2.0 | [github.com/petgraph/fixedbitset](https://github.com/petgraph/fixedbitset) |
| `flate2` | 1.1.10 | MIT OR Apache-2.0 | [github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| `fnv` | 1.0.7 | Apache-2.0 / MIT | [github.com/servo/rust-fnv](https://github.com/servo/rust-fnv) |
| `foldhash` | 0.1.5 | Zlib | [github.com/orlp/foldhash](https://github.com/orlp/foldhash) |
| `foldhash` | 0.2.0 | Zlib | [github.com/orlp/foldhash](https://github.com/orlp/foldhash) |
| `foreign-types` | 0.5.0 | MIT/Apache-2.0 | [github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 | [github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 | [github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 | [github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-executor` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-lite` | 2.6.1 | Apache-2.0 OR MIT | [github.com/smol-rs/futures-lite](https://github.com/smol-rs/futures-lite) |
| `futures-macro` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | [github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `gdk` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdk-pixbuf` | 0.18.5 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gdk-pixbuf-sys` | 0.18.0 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gdk-sys` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdkwayland-sys` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdkx11` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdkx11-sys` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `generic-array` | 0.14.7 | MIT | [github.com/fizyk20/generic-array](https://github.com/fizyk20/generic-array) |
| `gethostname` | 1.1.0 | Apache-2.0 | [codeberg.org/swsnr/gethostname.rs](https://codeberg.org/swsnr/gethostname.rs) |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 | [github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | [github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `gio` | 0.18.4 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gio-sys` | 0.18.1 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glib` | 0.18.5 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glib-macros` | 0.18.5 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glib-sys` | 0.18.1 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glob` | 0.3.4 | MIT OR Apache-2.0 | [github.com/rust-lang/glob](https://github.com/rust-lang/glob) |
| `gobject-sys` | 0.18.0 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gtk` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gtk-sys` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gtk3-macros` | 0.18.2 | MIT | [github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `half` | 2.7.1 | MIT OR Apache-2.0 | [github.com/VoidStarKat/half-rs](https://github.com/VoidStarKat/half-rs) |
| `hashbrown` | 0.12.3 | MIT OR Apache-2.0 | [github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | 0.15.5 | MIT OR Apache-2.0 | [github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | [github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `heck` | 0.4.1 | MIT OR Apache-2.0 | [github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| `heck` | 0.5.0 | MIT OR Apache-2.0 | [github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| `hermit-abi` | 0.5.3 | MIT OR Apache-2.0 | [github.com/hermit-os/hermit-rs](https://github.com/hermit-os/hermit-rs) |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | [github.com/KokaKiwi/rust-hex](https://github.com/KokaKiwi/rust-hex) |
| `html5ever` | 0.39.0 | MIT OR Apache-2.0 | [github.com/servo/html5ever](https://github.com/servo/html5ever) |
| `http` | 1.5.0 | MIT OR Apache-2.0 | [github.com/hyperium/http](https://github.com/hyperium/http) |
| `http-body` | 1.1.0 | MIT | [github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| `http-body-util` | 0.1.5 | MIT | [github.com/hyperium/http-body](https://github.com/hyperium/http-body) |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 | [github.com/seanmonstar/httparse](https://github.com/seanmonstar/httparse) |
| `hyper` | 1.11.1 | MIT | [github.com/hyperium/hyper](https://github.com/hyperium/hyper) |
| `hyper-util` | 0.1.21 | MIT | [github.com/hyperium/hyper-util](https://github.com/hyperium/hyper-util) |
| `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 | [github.com/strawlab/iana-time-zone](https://github.com/strawlab/iana-time-zone) |
| `iana-time-zone-haiku` | 0.1.2 | MIT OR Apache-2.0 | [github.com/strawlab/iana-time-zone](https://github.com/strawlab/iana-time-zone) |
| `ico` | 0.5.0 | MIT | [github.com/mdsteele/rust-ico](https://github.com/mdsteele/rust-ico) |
| `icu_collections` | 2.1.1 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_locale_core` | 2.1.1 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_normalizer` | 2.1.1 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_normalizer_data` | 2.1.1 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_properties` | 2.1.2 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_properties_data` | 2.1.2 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_provider` | 2.1.1 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `ident_case` | 1.0.1 | MIT/Apache-2.0 | [github.com/TedDriggs/ident_case](https://github.com/TedDriggs/ident_case) |
| `idna` | 1.1.0 | MIT OR Apache-2.0 | [github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| `idna_adapter` | 1.2.1 | Apache-2.0 OR MIT | [github.com/hsivonen/idna_adapter](https://github.com/hsivonen/idna_adapter) |
| `image` | 0.25.9 | MIT OR Apache-2.0 | [github.com/image-rs/image](https://github.com/image-rs/image) |
| `indexmap` | 1.9.3 | Apache-2.0 OR MIT | [github.com/bluss/indexmap](https://github.com/bluss/indexmap) |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT | [github.com/indexmap-rs/indexmap](https://github.com/indexmap-rs/indexmap) |
| `infer` | 0.22.0 | MIT | [github.com/bojand/infer](https://github.com/bojand/infer) |
| `ipnet` | 2.12.2 | MIT OR Apache-2.0 | [github.com/krisprice/ipnet](https://github.com/krisprice/ipnet) |
| `is-docker` | 0.2.0 | MIT | [github.com/TheLarkInn/is-docker](https://github.com/TheLarkInn/is-docker) |
| `is-wsl` | 0.4.0 | MIT | [github.com/TheLarkInn/is-wsl](https://github.com/TheLarkInn/is-wsl) |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | [github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| `javascriptcore-rs` | 1.1.2 | MIT | [github.com/tauri-apps/javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) |
| `javascriptcore-rs-sys` | 1.1.1 | MIT | [github.com/tauri-apps/javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) |
| `jni` | 0.21.1 | MIT/Apache-2.0 | [github.com/jni-rs/jni-rs](https://github.com/jni-rs/jni-rs) |
| `jni-sys` | 0.3.1 | MIT OR Apache-2.0 | [github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| `jni-sys` | 0.4.1 | MIT OR Apache-2.0 | [github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| `jni-sys-macros` | 0.4.1 | MIT OR Apache-2.0 | [github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| `js-sys` | 0.3.106 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| `json-patch` | 4.2.0 | MIT/Apache-2.0 | [github.com/idubrov/json-patch](https://github.com/idubrov/json-patch) |
| `jsonptr` | 0.7.1 | MIT OR Apache-2.0 | [github.com/chanced/jsonptr](https://github.com/chanced/jsonptr) |
| `keyboard-types` | 0.8.3 | MIT OR Apache-2.0 | [github.com/rust-windowing/keyboard-types](https://github.com/rust-windowing/keyboard-types) |
| `lazy_static` | 1.5.1 | MIT OR Apache-2.0 | [github.com/rust-lang-nursery/lazy-static.rs](https://github.com/rust-lang-nursery/lazy-static.rs) |
| `libappindicator` | 0.9.0 | Apache-2.0 OR MIT | — |
| `libappindicator-sys` | 0.9.0 | Apache-2.0 OR MIT | — |
| `libc` | 0.2.190 | MIT OR Apache-2.0 | [github.com/rust-lang/libc](https://github.com/rust-lang/libc) |
| `libdbus-sys` | 0.2.7 | Apache-2.0/MIT | [github.com/diwic/dbus-rs](https://github.com/diwic/dbus-rs) |
| `libloading` | 0.7.4 | ISC | [github.com/nagisa/rust_libloading/](https://github.com/nagisa/rust_libloading/) |
| `libredox` | 0.1.25 | MIT | [gitlab.redox-os.org/redox-os/libredox](https://gitlab.redox-os.org/redox-os/libredox) |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| `litemap` | 0.8.3 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 | [github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `log` | 0.4.34 | MIT OR Apache-2.0 | [github.com/rust-lang/log](https://github.com/rust-lang/log) |
| `markup5ever` | 0.39.0 | MIT OR Apache-2.0 | [github.com/servo/html5ever](https://github.com/servo/html5ever) |
| `matchers` | 0.2.0 | MIT | [github.com/hawkw/matchers](https://github.com/hawkw/matchers) |
| `memchr` | 2.8.3 | Unlicense OR MIT | [github.com/BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| `memoffset` | 0.9.1 | MIT | [github.com/Gilnaa/memoffset](https://github.com/Gilnaa/memoffset) |
| `mime` | 0.3.17 | MIT OR Apache-2.0 | [github.com/hyperium/mime](https://github.com/hyperium/mime) |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | [github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `miniz_oxide` | 0.9.1 | MIT OR Zlib OR Apache-2.0 | [github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `mio` | 1.2.3 | MIT | [github.com/tokio-rs/mio](https://github.com/tokio-rs/mio) |
| `moxcms` | 0.7.11 | BSD-3-Clause OR Apache-2.0 | [github.com/awxkee/moxcms](https://github.com/awxkee/moxcms) |
| `muda` | 0.20.0 | Apache-2.0 OR MIT | [github.com/tauri-apps/muda](https://github.com/tauri-apps/muda) |
| `ndk` | 0.9.0 | MIT OR Apache-2.0 | [github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| `ndk-context` | 0.1.1 | MIT OR Apache-2.0 | [github.com/rust-windowing/android-ndk-rs](https://github.com/rust-windowing/android-ndk-rs) |
| `ndk-sys` | 0.6.0+11769913 | MIT OR Apache-2.0 | [github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| `new_debug_unreachable` | 1.0.6 | MIT | [github.com/mbrubeck/rust-debug-unreachable](https://github.com/mbrubeck/rust-debug-unreachable) |
| `nom` | 8.0.0 | MIT | [github.com/rust-bakery/nom](https://github.com/rust-bakery/nom) |
| `nu-ansi-term` | 0.50.3 | MIT | [github.com/nushell/nu-ansi-term](https://github.com/nushell/nu-ansi-term) |
| `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | [github.com/illicitonion/num_enum](https://github.com/illicitonion/num_enum) |
| `num_enum_derive` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | [github.com/illicitonion/num_enum](https://github.com/illicitonion/num_enum) |
| `num-conv` | 0.1.0 | MIT OR Apache-2.0 | [github.com/jhpratt/num-conv](https://github.com/jhpratt/num-conv) |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | [github.com/rust-num/num-traits](https://github.com/rust-num/num-traits) |
| `objc2` | 0.6.4 | MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-cloud-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-data` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-image` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-location` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-text` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-encode` | 4.1.0 | MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-exception-helper` | 0.1.1 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-foundation` | 0.3.2 | MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-quartz-core` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-ui-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-user-notifications` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-web-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | [github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | [github.com/matklad/once_cell](https://github.com/matklad/once_cell) |
| `open` | 5.4.4 | MIT | [github.com/Byron/open-rs](https://github.com/Byron/open-rs) |
| `option-ext` | 0.2.0 | MPL-2.0 | [github.com/soc/option-ext](https://github.com/soc/option-ext) |
| `ordered-stream` | 0.2.0 | MIT OR Apache-2.0 | [github.com/danieldg/ordered-stream](https://github.com/danieldg/ordered-stream) |
| `os_pipe` | 1.2.3 | MIT | [github.com/oconnor663/os_pipe.rs](https://github.com/oconnor663/os_pipe.rs) |
| `pango` | 0.18.3 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `pango-sys` | 0.18.0 | MIT | [github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `parking` | 2.2.1 | Apache-2.0 OR MIT | [github.com/smol-rs/parking](https://github.com/smol-rs/parking) |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | [github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | [github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | [github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| `petgraph` | 0.8.3 | MIT OR Apache-2.0 | [github.com/petgraph/petgraph](https://github.com/petgraph/petgraph) |
| `phf` | 0.13.1 | MIT | [github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `phf_generator` | 0.13.1 | MIT | [github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `phf_macros` | 0.13.1 | MIT | [github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `phf_shared` | 0.13.1 | MIT | [github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | [github.com/taiki-e/pin-project-lite](https://github.com/taiki-e/pin-project-lite) |
| `piper` | 0.2.5 | MIT OR Apache-2.0 | [github.com/smol-rs/piper](https://github.com/smol-rs/piper) |
| `plist` | 1.8.0 | MIT | [github.com/ebarnard/rust-plist/](https://github.com/ebarnard/rust-plist/) |
| `png` | 0.17.16 | MIT OR Apache-2.0 | [github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| `png` | 0.18.1 | MIT OR Apache-2.0 | [github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| `polling` | 3.11.0 | Apache-2.0 OR MIT | [github.com/smol-rs/polling](https://github.com/smol-rs/polling) |
| `potential_utf` | 0.1.6 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `powerfmt` | 0.2.0 | MIT OR Apache-2.0 | [github.com/jhpratt/powerfmt](https://github.com/jhpratt/powerfmt) |
| `precomputed-hash` | 0.1.1 | MIT | [github.com/emilio/precomputed-hash](https://github.com/emilio/precomputed-hash) |
| `proc-macro-crate` | 1.3.1 | MIT OR Apache-2.0 | [github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-crate` | 2.0.2 | MIT OR Apache-2.0 | [github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 | [github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-error` | 1.0.4 | MIT OR Apache-2.0 | [gitlab.com/CreepySkeleton/proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| `proc-macro-error-attr` | 1.0.4 | MIT OR Apache-2.0 | [gitlab.com/CreepySkeleton/proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | [github.com/dtolnay/proc-macro2](https://github.com/dtolnay/proc-macro2) |
| `pxfm` | 0.1.30 | BSD-3-Clause OR Apache-2.0 | [github.com/awxkee/pxfm](https://github.com/awxkee/pxfm) |
| `quick-error` | 2.0.1 | MIT/Apache-2.0 | [github.com/tailhook/quick-error](http://github.com/tailhook/quick-error) |
| `quick-xml` | 0.38.4 | MIT | [github.com/tafia/quick-xml](https://github.com/tafia/quick-xml) |
| `quick-xml` | 0.41.0 | MIT | [github.com/tafia/quick-xml](https://github.com/tafia/quick-xml) |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | [github.com/dtolnay/quote](https://github.com/dtolnay/quote) |
| `r-efi` | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | [github.com/rust-windowing/raw-window-handle](https://github.com/rust-windowing/raw-window-handle) |
| `redox_syscall` | 0.5.18 | MIT | [gitlab.redox-os.org/redox-os/syscall](https://gitlab.redox-os.org/redox-os/syscall) |
| `redox_users` | 0.5.3 | MIT | [gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| `ref-cast` | 1.0.27 | MIT OR Apache-2.0 | [github.com/dtolnay/ref-cast](https://github.com/dtolnay/ref-cast) |
| `ref-cast-impl` | 1.0.27 | MIT OR Apache-2.0 | [github.com/dtolnay/ref-cast](https://github.com/dtolnay/ref-cast) |
| `regex` | 1.13.1 | MIT OR Apache-2.0 | [github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 | [github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 | [github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `reqwest` | 0.13.5 | MIT OR Apache-2.0 | [github.com/seanmonstar/reqwest](https://github.com/seanmonstar/reqwest) |
| `rfd` | 0.16.0 | MIT | [github.com/PolyMeilex/rfd](https://github.com/PolyMeilex/rfd) |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | [github.com/rust-lang/rustc-hash](https://github.com/rust-lang/rustc-hash) |
| `rustix` | 1.1.5 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | [github.com/dtolnay/rustversion](https://github.com/dtolnay/rustversion) |
| `same-file` | 1.0.6 | Unlicense/MIT | [github.com/BurntSushi/same-file](https://github.com/BurntSushi/same-file) |
| `schemars` | 0.8.22 | MIT | [github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| `schemars` | 0.9.0 | MIT | [github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| `schemars` | 1.2.2 | MIT | [github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| `schemars_derive` | 0.8.22 | MIT | [github.com/GREsau/schemars](https://github.com/GREsau/schemars) |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | [github.com/bluss/scopeguard](https://github.com/bluss/scopeguard) |
| `selectors` | 0.38.0 | MPL-2.0 | [github.com/servo/stylo](https://github.com/servo/stylo) |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | [github.com/dtolnay/semver](https://github.com/dtolnay/semver) |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_derive_internals` | 0.29.1 | MIT OR Apache-2.0 | [github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_json` | 1.0.151 | MIT OR Apache-2.0 | [github.com/serde-rs/json](https://github.com/serde-rs/json) |
| `serde_repr` | 0.1.21 | MIT OR Apache-2.0 | [github.com/dtolnay/serde-repr](https://github.com/dtolnay/serde-repr) |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `serde_spanned` | 1.1.1 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `serde_with` | 3.17.0 | MIT OR Apache-2.0 | [github.com/jonasbb/serde_with/](https://github.com/jonasbb/serde_with/) |
| `serde_with_macros` | 3.17.0 | MIT OR Apache-2.0 | [github.com/jonasbb/serde_with/](https://github.com/jonasbb/serde_with/) |
| `serde-untagged` | 0.1.9 | MIT OR Apache-2.0 | [github.com/dtolnay/serde-untagged](https://github.com/dtolnay/serde-untagged) |
| `serialize-to-javascript` | 0.1.2 | MIT OR Apache-2.0 | [github.com/chippers/serialize-to-javascript](https://github.com/chippers/serialize-to-javascript) |
| `serialize-to-javascript-impl` | 0.1.2 | MIT OR Apache-2.0 | [github.com/chippers/serialize-to-javascript](https://github.com/chippers/serialize-to-javascript) |
| `servo_arc` | 0.4.3 | MIT OR Apache-2.0 | [github.com/servo/stylo](https://github.com/servo/stylo) |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | [github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `sharded-slab` | 0.1.7 | MIT | [github.com/hawkw/sharded-slab](https://github.com/hawkw/sharded-slab) |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 | [github.com/vorner/signal-hook](https://github.com/vorner/signal-hook) |
| `simd-adler32` | 0.3.10 | MIT | [github.com/mcountryman/simd-adler32](https://github.com/mcountryman/simd-adler32) |
| `siphasher` | 1.0.4 | MIT OR Apache-2.0 | [github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| `slab` | 0.4.12 | MIT | [github.com/tokio-rs/slab](https://github.com/tokio-rs/slab) |
| `smallvec` | 1.16.2 | MIT OR Apache-2.0 | [github.com/servo/rust-smallvec](https://github.com/servo/rust-smallvec) |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 | [github.com/rust-lang/socket2](https://github.com/rust-lang/socket2) |
| `softbuffer` | 0.4.8 | MIT OR Apache-2.0 | [github.com/rust-windowing/softbuffer](https://github.com/rust-windowing/softbuffer) |
| `soup3` | 0.5.0 | MIT | [gitlab.gnome.org/World/Rust/soup3-rs](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| `soup3-sys` | 0.5.0 | MIT | [gitlab.gnome.org/World/Rust/soup3-rs](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | [github.com/storyyeller/stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) |
| `string_cache` | 0.9.0 | MIT OR Apache-2.0 | [github.com/servo/string-cache](https://github.com/servo/string-cache) |
| `strsim` | 0.11.1 | MIT | [github.com/rapidfuzz/strsim-rs](https://github.com/rapidfuzz/strsim-rs) |
| `swift-rs` | 1.0.8 | MIT OR Apache-2.0 | [github.com/Brendonovich/swift-rs](https://github.com/Brendonovich/swift-rs) |
| `syn` | 1.0.109 | MIT OR Apache-2.0 | [github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | [github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `syn` | 3.0.6 | MIT OR Apache-2.0 | [github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `sync_wrapper` | 1.0.2 | Apache-2.0 | [github.com/Actyx/sync_wrapper](https://github.com/Actyx/sync_wrapper) |
| `synstructure` | 0.14.0 | MIT | [github.com/mystor/synstructure](https://github.com/mystor/synstructure) |
| `tao` | 0.37.1 | Apache-2.0 | [github.com/tauri-apps/tao](https://github.com/tauri-apps/tao) |
| `tao-macros` | 0.1.4 | MIT OR Apache-2.0 | [github.com/tauri-apps/tao](https://github.com/tauri-apps/tao) |
| `tauri` | 2.12.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `tauri-codegen` | 2.7.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `tauri-macros` | 2.7.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `tauri-plugin-clipboard-manager` | 2.4.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| `tauri-plugin-dialog` | 2.8.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| `tauri-plugin-fs` | 2.6.0 | Apache-2.0 OR MIT | [github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| `tauri-plugin-opener` | 2.7.0 | Apache-2.0 OR MIT | [github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| `tauri-plugin-single-instance` | 2.5.4 | Apache-2.0 OR MIT | [github.com/tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) |
| `tauri-runtime` | 2.12.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `tauri-runtime-wry` | 2.12.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `tauri-utils` | 2.10.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri](https://github.com/tauri-apps/tauri) |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | [github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| `tendril` | 0.5.1 | MIT OR Apache-2.0 | [github.com/servo/html5ever](https://github.com/servo/html5ever) |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | 2.0.21 | MIT OR Apache-2.0 | [github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thread_local` | 1.1.10 | MIT OR Apache-2.0 | [github.com/Amanieu/thread_local-rs](https://github.com/Amanieu/thread_local-rs) |
| `tiff` | 0.10.3 | MIT | [github.com/image-rs/image-tiff](https://github.com/image-rs/image-tiff) |
| `time` | 0.3.45 | MIT OR Apache-2.0 | [github.com/time-rs/time](https://github.com/time-rs/time) |
| `time-core` | 0.1.7 | MIT OR Apache-2.0 | [github.com/time-rs/time](https://github.com/time-rs/time) |
| `time-macros` | 0.2.25 | MIT OR Apache-2.0 | [github.com/time-rs/time](https://github.com/time-rs/time) |
| `tinystr` | 0.8.4 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `tokio` | 1.53.1 | MIT | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `tokio-macros` | 2.7.2 | MIT | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `tokio-util` | 0.7.19 | MIT | [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `toml` | 1.1.6+spec-1.1.0 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_datetime` | 0.6.3 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_datetime` | 1.1.1+spec-1.1.0 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.19.15 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.20.2 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | 0.25.15+spec-1.1.0 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_parser` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_writer` | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | [github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `tower` | 0.5.3 | MIT | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| `tower-http` | 0.6.11 | MIT | [github.com/tower-rs/tower-http](https://github.com/tower-rs/tower-http) |
| `tower-layer` | 0.3.3 | MIT | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| `tower-service` | 0.3.3 | MIT | [github.com/tower-rs/tower](https://github.com/tower-rs/tower) |
| `tracing` | 0.1.44 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-attributes` | 0.1.31 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-core` | 0.1.36 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-log` | 0.2.0 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-subscriber` | 0.3.23 | MIT | [github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tray-icon` | 0.25.1 | MIT OR Apache-2.0 | [github.com/tauri-apps/tray-icon](https://github.com/tauri-apps/tray-icon) |
| `tree_magic_mini` | 3.2.2 | MIT | [github.com/mbrubeck/tree_magic/](https://github.com/mbrubeck/tree_magic/) |
| `try-lock` | 0.2.5 | MIT | [github.com/seanmonstar/try-lock](https://github.com/seanmonstar/try-lock) |
| `typeid` | 1.0.3 | MIT OR Apache-2.0 | [github.com/dtolnay/typeid](https://github.com/dtolnay/typeid) |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | [github.com/paholg/typenum](https://github.com/paholg/typenum) |
| `uds_windows` | 1.2.1 | MIT | [github.com/haraldh/rust_uds_windows](https://github.com/haraldh/rust_uds_windows) |
| `unicode-ident` | 1.0.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | [github.com/dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 | [github.com/unicode-rs/unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) |
| `url` | 2.5.8 | MIT OR Apache-2.0 | [github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `urlpattern` | 0.6.0 | MIT | [github.com/denoland/rust-urlpattern](https://github.com/denoland/rust-urlpattern) |
| `utf8_iter` | 1.0.4 | Apache-2.0 OR MIT | [github.com/hsivonen/utf8_iter](https://github.com/hsivonen/utf8_iter) |
| `uuid` | 1.26.1 | Apache-2.0 OR MIT | [github.com/uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| `valuable` | 0.1.1 | MIT | [github.com/tokio-rs/valuable](https://github.com/tokio-rs/valuable) |
| `walkdir` | 2.5.0 | Unlicense/MIT | [github.com/BurntSushi/walkdir](https://github.com/BurntSushi/walkdir) |
| `want` | 0.3.1 | MIT | [github.com/seanmonstar/want](https://github.com/seanmonstar/want) |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| `wasip2` | 1.0.1+wasi-0.2.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/wasi-rs](https://github.com/bytecodealliance/wasi-rs) |
| `wasm-bindgen` | 0.2.129 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) |
| `wasm-bindgen-futures` | 0.4.79 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| `wasm-bindgen-macro` | 0.2.129 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| `wasm-bindgen-macro-support` | 0.2.129 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support](https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support) |
| `wasm-bindgen-shared` | 0.2.129 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| `wasm-streams` | 0.5.0 | MIT OR Apache-2.0 | [github.com/MattiasBuelens/wasm-streams/](https://github.com/MattiasBuelens/wasm-streams/) |
| `wayland-backend` | 0.3.17 | MIT | [github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-client` | 0.31.15 | MIT | [github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols` | 0.32.12 | MIT | [github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-wlr` | 0.3.12 | MIT | [github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-scanner` | 0.31.11 | MIT | [github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-sys` | 0.31.11 | MIT | [github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `web_atoms` | 0.2.6 | MIT OR Apache-2.0 | [github.com/servo/html5ever](https://github.com/servo/html5ever) |
| `web-sys` | 0.3.106 | MIT OR Apache-2.0 | [github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 | [github.com/daxpedda/web-time](https://github.com/daxpedda/web-time) |
| `webkit2gtk` | 2.0.2 | MIT | [github.com/tauri-apps/webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) |
| `webkit2gtk-sys` | 2.0.2 | MIT | [github.com/tauri-apps/webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) |
| `webview2-com` | 0.39.1 | MIT | [github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| `webview2-com-macros` | 0.8.1 | MIT | [github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| `webview2-com-sys` | 0.39.1 | MIT | [github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| `weezl` | 0.1.12 | MIT OR Apache-2.0 | [github.com/image-rs/weezl](https://github.com/image-rs/weezl) |
| `winapi` | 0.3.9 | MIT/Apache-2.0 | [github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| `winapi-i686-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | [github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| `winapi-util` | 0.1.11 | Unlicense OR MIT | [github.com/BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util) |
| `winapi-x86_64-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | [github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| `window-vibrancy` | 0.8.1 | Apache-2.0 OR MIT | [github.com/tauri-apps/tauri-plugin-vibrancy](https://github.com/tauri-apps/tauri-plugin-vibrancy) |
| `windows` | 0.62.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnullvm` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | 0.53.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-collections` | 0.3.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-core` | 0.62.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-future` | 0.3.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-numerics` | 0.3.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-result` | 0.4.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-strings` | 0.5.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.45.0 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.60.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.42.2 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-targets` | 0.53.5 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-threading` | 0.2.1 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-version` | 0.1.7 | MIT OR Apache-2.0 | [github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `winnow` | 0.5.40 | MIT | [github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| `winnow` | 0.7.15 | MIT | [github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| `winnow` | 1.0.4 | MIT | [github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| `wit-bindgen` | 0.46.0 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) |
| `wl-clipboard-rs` | 0.9.4 | MIT OR Apache-2.0 | [github.com/YaLTeR/wl-clipboard-rs](https://github.com/YaLTeR/wl-clipboard-rs) |
| `writeable` | 0.6.4 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `wry` | 0.57.0 | Apache-2.0 OR MIT | [github.com/tauri-apps/wry](https://github.com/tauri-apps/wry) |
| `x11` | 2.21.0 | MIT | [github.com/AltF02/x11-rs](https://github.com/AltF02/x11-rs) |
| `x11-dl` | 2.21.0 | MIT | [github.com/AltF02/x11-rs](https://github.com/AltF02/x11-rs) |
| `x11rb` | 0.13.2 | MIT OR Apache-2.0 | [github.com/psychon/x11rb](https://github.com/psychon/x11rb) |
| `x11rb-protocol` | 0.13.2 | MIT OR Apache-2.0 | [github.com/psychon/x11rb](https://github.com/psychon/x11rb) |
| `yoke` | 0.8.3 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `yoke-derive` | 0.8.4 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zbus` | 5.13.2 | MIT | [github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zbus_macros` | 5.13.2 | MIT | [github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zbus_names` | 4.3.1 | MIT | [github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zerocopy` | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT | [github.com/google/zerocopy](https://github.com/google/zerocopy) |
| `zerocopy-derive` | 0.8.59 | BSD-2-Clause OR Apache-2.0 OR MIT | [github.com/google/zerocopy](https://github.com/google/zerocopy) |
| `zerofrom` | 0.1.8 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerofrom-derive` | 0.1.8 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerotrie` | 0.2.5 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerovec` | 0.11.8 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerovec-derive` | 0.11.6 | Unicode-3.0 | [github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zlib-rs` | 0.6.8 | Zlib | [github.com/trifectatechfoundation/zlib-rs](https://github.com/trifectatechfoundation/zlib-rs) |
| `zmij` | 1.0.23 | MIT | [github.com/dtolnay/zmij](https://github.com/dtolnay/zmij) |
| `zune-core` | 0.4.12 | MIT OR Apache-2.0 OR Zlib | — |
| `zune-jpeg` | 0.4.21 | MIT OR Apache-2.0 OR Zlib | [github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg](https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg) |
| `zvariant` | 5.9.2 | MIT | [github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zvariant_derive` | 5.9.2 | MIT | [github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zvariant_utils` | 3.3.0 | MIT | [github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |

## Where the full license texts are

The tables give the SPDX identifier of each component. Each component ships its own copy of
the license (the Repository column points at it), and the canonical texts are published by
SPDX:

- MIT — <https://spdx.org/licenses/MIT.html>
- Apache-2.0 — <https://spdx.org/licenses/Apache-2.0.html>
- Zlib — <https://spdx.org/licenses/Zlib.html>
- Unicode-3.0 — <https://spdx.org/licenses/Unicode-3.0.html>
- BSD-3-Clause — <https://spdx.org/licenses/BSD-3-Clause.html>
- Unlicense — <https://spdx.org/licenses/Unlicense.html>
- Apache-2.0 WITH LLVM-exception — <https://spdx.org/licenses/Apache-2.0.html> with <https://spdx.org/licenses/LLVM-exception.html>
- MPL-2.0 — <https://spdx.org/licenses/MPL-2.0.html>
- BSD-2-Clause — <https://spdx.org/licenses/BSD-2-Clause.html>
- ISC — <https://spdx.org/licenses/ISC.html>
- BSL-1.0 — <https://spdx.org/licenses/BSL-1.0.html>
- LGPL-2.1-or-later — <https://spdx.org/licenses/LGPL-2.1-or-later.html>
- 0BSD — <https://spdx.org/licenses/0BSD.html>
- CC0-1.0 — <https://spdx.org/licenses/CC0-1.0.html>
- MIT-0 — <https://spdx.org/licenses/MIT-0.html>
