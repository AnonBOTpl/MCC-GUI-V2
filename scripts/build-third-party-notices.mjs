#!/usr/bin/env node
// Buduje `THIRD-PARTY-NOTICES.md` z wyników dwóch narzędzi wydaniowych:
//
//   npx --yes license-checker-rseidelsohn@5.0.1 --production --json --start . > npm.json
//   cd src-tauri && cargo bundle-licenses --format yaml --output THIRDPARTY.yml
//   node scripts/build-third-party-notices.mjs npm.json src-tauri/THIRDPARTY.yml
//
// Skrypt czyta tylko te dwa pliki (nic nie instaluje i nie uruchamia) i wypisuje listę
// komponentów: nazwa, wersja, licencja, adres repozytorium, plus zestawienie licencji.
// Kolejność argumentów: <npm.json> <rust.yml> [wyjście].

import { readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

/** Nazwa własnego pakietu - nie jest komponentem osoby trzeciej. */
const OWN_PACKAGE = 'mcc-gui-v2'

/** Katalog główny projektu (skrypt leży w `scripts/`). */
const PROJECT_ROOT = dirname(dirname(fileURLToPath(import.meta.url)))

/**
 * Parsuje YAML z `cargo-bundle-licenses`. Format jest regularny (jedna sekcja na skrzynię),
 * więc nie ciągniemy do tego zależności `js-yaml`.
 */
export function parseBundleLicenses(path) {
  const lines = readFileSync(path, 'utf8').split(/\r?\n/)
  const libraries = []
  let library = null
  let current = null
  let inText = false
  let textIndent = 0

  /** Zamienia zebrane wiersze na tekst (blok bez `text: |` zostaje pusty). */
  const textOf = (value) => {
    if (typeof value === 'string') return value
    if (!Array.isArray(value)) return ''
    return value.join('\n').replace(/\n+$/, '')
  }
  const closeText = () => {
    if (inText && current) current.text = textOf(current.text)
    inText = false
  }
  const closeLicense = () => {
    closeText()
    if (library && current) library.licenses.push(current.id)
    current = null
  }

  for (const line of lines) {
    if (line.startsWith('- package_name: ')) {
      closeLicense()
      library = {
        name: line.slice('- package_name: '.length).trim(),
        version: '',
        repository: '',
        license: '',
        licenses: [],
      }
      libraries.push(library)
      continue
    }
    if (!library) continue

    if (inText) {
      const indent = line.length - line.trimStart().length
      if (line.trim() === '' || indent >= textIndent) {
        current.text.push(line.slice(textIndent))
        continue
      }
      closeText()
    }

    if (line.startsWith('  package_version: ')) {
      library.version = line.slice('  package_version: '.length).trim()
    } else if (line.startsWith('  repository: ')) {
      library.repository = line.slice('  repository: '.length).trim()
    } else if (line.startsWith('  license: ') && !line.startsWith('  license: |')) {
      library.license = line.slice('  license: '.length).trim()
    } else if (line.startsWith('  - license: ')) {
      closeLicense()
      current = { id: line.slice('  - license: '.length).trim(), text: [] }
    } else if (line.trim() === 'text: |' && current) {
      inText = true
      textIndent = line.length - line.trimStart().length + 2
      current.text = []
    }
  }
  closeLicense()
  return libraries
}

/** Rozbija wyrażenie licencyjne na pojedyncze identyfikatory (OR / AND / ukośnik). */
export function licenseIds(expression) {
  return (expression || '')
    .split(/\s+(?:OR|AND)\s+|\//)
    .map((part) => part.trim().replace(/^\(|\)$/g, '').trim())
    .filter(Boolean)
}

/** Adres repozytorium w jednej formie (npm podaje obiekt, Cargo czasem listę). */
function repositoryOf(value) {
  let url = ''
  if (typeof value === 'string') url = value
  else if (value && typeof value === 'object') url = value.url ?? ''
  url = url
    .replace(/^git\+/, '')
    .replace(/\.git$/, '')
    .trim()
  // Nie każdy wpis jest adresem (np. `repository: ['']` w Cargo.toml).
  return /^https?:\/\//.test(url) ? url : ''
}

/** Wiersz tabeli: komponent | wersja | licencja | repozytorium. */
function tableRow(row) {
  const repository = row.repository
    ? '[' + row.repository.replace(/^https?:\/\//, '') + '](' + row.repository + ')'
    : '—'
  return '| `' + row.name + '` | ' + row.version + ' | ' + row.license + ' | ' + repository + ' |'
}

function table(rows) {
  return ['| Component | Version | License | Repository |', '|---|---|---|---|']
    .concat(rows.map(tableRow))
    .join('\n')
}

/** Link do tekstu licencji w SPDX (obsługa identyfikatora z wyjątkiem `WITH`). */
export function spdxLink(id) {
  const parts = id.split(/\s+WITH\s+/)
  if (parts.length === 2) {
    return (
      `<https://spdx.org/licenses/${parts[0]}.html>` +
      ` with <https://spdx.org/licenses/${parts[1]}.html>`
    )
  }
  return `<https://spdx.org/licenses/${id}.html>`
}

/** Zestawienie identyfikatorów licencji z liczbą komponentów po każdej stronie. */
export function breakdown(npmPackages, rustCrates) {
  const npmCounts = new Map()
  const rustCounts = new Map()
  for (const item of npmPackages) {
    for (const id of new Set(licenseIds(item.license))) {
      npmCounts.set(id, (npmCounts.get(id) ?? 0) + 1)
    }
  }
  for (const item of rustCrates) {
    for (const id of new Set(licenseIds(item.license))) {
      rustCounts.set(id, (rustCounts.get(id) ?? 0) + 1)
    }
  }
  const all = new Set([...npmCounts.keys(), ...rustCounts.keys()])
  const rows = [...all]
    .map((id) => ({ id, npm: npmCounts.get(id) ?? 0, rust: rustCounts.get(id) ?? 0 }))
    .sort((a, b) => b.npm + b.rust - (a.npm + a.rust) || a.id.localeCompare(b.id))
  return {
    rows,
    text: ['| License | npm packages | Rust crates |', '|---|---|---|']
      .concat(rows.map((row) => `| ${row.id} | ${row.npm || '—'} | ${row.rust || '—'} |`))
      .join('\n'),
  }
}

/** Składa cały dokument. */
export function buildNotice(npmPackages, rustCrates, generatedOn, version) {
  const { rows: licenseRows, text: breakdownTable } = breakdown(npmPackages, rustCrates)

  const lines = []
  lines.push('# Third-party notices')
  lines.push('')
  lines.push(
    'MCC GUI V2 is released under the **MIT License** — see [LICENSE](LICENSE). It builds on the',
  )
  lines.push(
    'third-party components listed here; this file records them and the license each one is under.',
  )
  lines.push('')
  lines.push(`Generated on **${generatedOn}** for version **${version}**, from the versions locked in`)
  lines.push(
    '`package-lock.json` and `src-tauri/Cargo.lock` — the same inputs the Windows installer was',
  )
  lines.push('built from.')
  lines.push('')
  lines.push('## Scope')
  lines.push('')
  lines.push(
    `- **JavaScript** (${npmPackages.length} packages): the **production** dependency tree of`,
  )
  lines.push(
    '  `package.json`, including everything those packages pull in. Build tools (Vite, ESLint,',
  )
  lines.push('  Tailwind, Vitest) are not part of it, because they never end up in the app.')
  lines.push(
    `- **Rust** (${rustCrates.length} crates): everything resolved from \`src-tauri/Cargo.lock\`.`,
  )
  lines.push('  That resolution is platform-wide, so a handful of crates that only build on other')
  lines.push('  platforms are listed too, even though they are not compiled into the Windows binary.')
  lines.push('')
  lines.push('## How to regenerate this file')
  lines.push('')
  lines.push('```bash')
  lines.push('# npm, production tree only (license-checker-rseidelsohn 5.0.1)')
  lines.push('npx --yes license-checker-rseidelsohn@5.0.1 --production --json --start . > npm.json')
  lines.push('')
  lines.push('# Rust crates, together with the license texts (cargo-bundle-licenses 4.2.0)')
  lines.push('cd src-tauri && cargo bundle-licenses --format yaml --output THIRDPARTY.yml && cd ..')
  lines.push('')
  lines.push('# and the file itself')
  lines.push('node scripts/build-third-party-notices.mjs npm.json src-tauri/THIRDPARTY.yml')
  lines.push('```')
  lines.push('')
  lines.push('## Licenses in use')
  lines.push('')
  lines.push(breakdownTable)
  lines.push('')
  lines.push(
    'No component here is under a strong-copyleft license (GPL or AGPL), and none of them is',
  )
  lines.push(
    'modified: the app only links against them as published. The licenses worth naming explicitly:',
  )
  lines.push('')
  lines.push(
    '- **MPL-2.0** — `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext` and `selectors`.',
  )
  lines.push(
    "  MPL-2.0 is file-level copyleft: it covers those crates' own source files and explicitly",
  )
  lines.push(
    '  allows them to be combined with code under other licenses, which is what happens here. The',
  )
  lines.push('  crates are used unmodified and their sources stay available upstream (Repository column).')
  lines.push('- **LGPL-2.1-or-later** — appears only in `r-efi`, which is licensed')
  lines.push(
    '  `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; this project takes the MIT/Apache-2.0 option.',
  )
  lines.push(
    '- **Unlicense**, **CC0-1.0**, **MIT-0** and **0BSD** are permissive to the point of asking for',
  )
  lines.push(
    '  no attribution; they are listed here for completeness. **BSL-1.0** (Boost) is permissive as',
  )
  lines.push(
    '  well, but asks for its notice to be kept with copies — the components under it are in the',
  )
  lines.push('  table below, with links to their upstream sources.')
  lines.push('')
  lines.push(
    '**Minecraft Console Client is not on this list.** It is not a dependency of this project and',
  )
  lines.push(
    'is never bundled or redistributed with it — the app only starts the `MinecraftClient.exe` you',
  )
  lines.push('point it at, as a separate process. See the licence section of the README.')
  lines.push('')
  lines.push('## JavaScript packages')
  lines.push('')
  lines.push(table(npmPackages))
  lines.push('')
  lines.push('## Rust crates')
  lines.push('')
  lines.push(table(rustCrates))
  lines.push('')
  lines.push('## Where the full license texts are')
  lines.push('')
  lines.push(
    'The tables give the SPDX identifier of each component. Each component ships its own copy of',
  )
  lines.push(
    'the license (the Repository column points at it), and the canonical texts are published by',
  )
  lines.push('SPDX:')
  lines.push('')
  for (const row of licenseRows) lines.push(`- ${row.id} — ${spdxLink(row.id)}`)
  lines.push('')
  return lines.join('\n')
}

const isMain = process.argv[1] !== undefined && import.meta.url === `file:///${process.argv[1].replace(/\\/g, '/')}`

if (isMain) {
  const [, , npmPath = 'npm.json', rustPath = 'src-tauri/THIRDPARTY.yml', output = 'THIRD-PARTY-NOTICES.md'] =
    process.argv

  const npm = JSON.parse(readFileSync(npmPath, 'utf8'))
  const npmPackages = Object.entries(npm)
    .filter(([key]) => !key.startsWith(OWN_PACKAGE + '@'))
    .map(([key, info]) => {
      const at = key.lastIndexOf('@')
      return {
        name: key.slice(0, at),
        version: key.slice(at + 1),
        license: info.licenses ?? '',
        repository: repositoryOf(info.repository),
      }
    })
    .sort((a, b) => a.name.localeCompare(b.name))

  const rustCrates = parseBundleLicenses(rustPath)
    .map((crate) => ({
      name: crate.name,
      version: crate.version,
      license: crate.license,
      repository: repositoryOf(crate.repository),
    }))
    .sort((a, b) => a.name.localeCompare(b.name))

  // Data i wersja biorą się z otoczenia, żeby nagłówek nie kłamał po podniesieniu numeru.
  const generatedOn = new Date().toISOString().slice(0, 10)
  const { version } = JSON.parse(readFileSync(join(PROJECT_ROOT, 'package.json'), 'utf8'))
  writeFileSync(output, buildNotice(npmPackages, rustCrates, generatedOn, version), 'utf8')
  console.log(`zapisano ${output}`)
  console.log(`npm: ${npmPackages.length}, rust: ${rustCrates.length}`)
  console.log(`data w nagłówku: ${generatedOn} (popraw ręcznie, jeśli generujesz z opóźnieniem)`)
}
