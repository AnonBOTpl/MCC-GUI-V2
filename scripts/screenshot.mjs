#!/usr/bin/env node
// Zrzuty prawdziwego okna aplikacji Tauri przez Chrome DevTools Protocol, bez
// dotykania klawiatury ani myszy.
//
//   node scripts/screenshot.mjs plan.json
//
// Plan (JSON) obsługuje m.in.:
//   - width/height/scale   - dokładny widok (px CSS) i deviceScaleFactor,
//   - wait                 - ile ms poczekać po `prepare`,
//   - prepare              - JS wykonany przed zdjęciem (klikanie, wybór zakładki),
//   - measure              - JS mierzący stan (wypisany na konsolę),
//   - clip / clipTo        - przycięcie kadru (clipTo: wyrażenie zwracające prostokąt),
//   - capture: false       - samo zmierzenie stanu, bez zapisu pliku.
//
// Wymaga portu debug 9222 - patrz nagłówek `scripts/ui-drive.mjs`.

import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'

const ENDPOINT = process.env.CDP_ENDPOINT ?? 'http://127.0.0.1:9222'
const planPath = process.argv[2]

if (!planPath) {
  console.error('Użycie: node scripts/screenshot.mjs plan.json')
  process.exit(2)
}

const plan = JSON.parse(readFileSync(planPath, 'utf8'))
const outDir = plan.out ?? '.'
const defaultWidth = plan.width ?? 1000
const defaultHeight = plan.height ?? 640
const defaultScale = plan.scale ?? 1

const targets = await fetch(`${ENDPOINT}/json/list`).then((response) => response.json())
const page = targets.find((target) => target.type === 'page')
if (!page?.webSocketDebuggerUrl) {
  console.error("Nie znaleziono targetu typu 'page' - czy aplikacja działa z debug portem?")
  process.exit(2)
}

const socket = new WebSocket(page.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  socket.onopen = resolve
  socket.onerror = () => reject(new Error('nie udało się otworzyć WebSocket do CDP'))
})

let nextId = 1
const pending = new Map()

socket.onmessage = (event) => {
  const message = JSON.parse(event.data)
  if (message.id === undefined) return // zdarzenia nas nie interesują
  const entry = pending.get(message.id)
  if (!entry) return
  pending.delete(message.id)
  if (message.error) entry.reject(new Error(`${message.error.message} (kod ${message.error.code})`))
  else entry.resolve(message.result)
}

function send(method, params = {}) {
  const id = nextId++
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject })
    socket.send(JSON.stringify({ id, method, params }))
  })
}

// Sygnatura PNG + wymiary z nagłówka IHDR, żeby od razu widzieć, czy `clip` i skala zadziałały.
function pngSize(buffer) {
  const isPng = buffer.subarray(1, 4).toString('latin1') === 'PNG'
  return { png: isPng, width: buffer.readUInt32BE(16), height: buffer.readUInt32BE(20) }
}

await send('Page.enable')
mkdirSync(outDir, { recursive: true })

for (const shot of plan.shots) {
  const width = shot.width ?? defaultWidth
  const height = shot.height ?? defaultHeight
  const scale = shot.scale ?? defaultScale

  await send('Emulation.setDeviceMetricsOverride', {
    width,
    height,
    deviceScaleFactor: scale,
    mobile: false,
  })

  if (shot.prepare) {
    const prepared = await send('Runtime.evaluate', {
      expression: shot.prepare,
      awaitPromise: true,
      returnByValue: true,
      userGesture: true,
    })
    if (prepared.exceptionDetails) {
      console.error(`[${shot.name}] WYJĄTEK w prepare: ${prepared.exceptionDetails.text}`)
      process.exitCode = 1
    } else if (prepared.result?.value !== undefined) {
      console.log(`[${shot.name}] stan: ${prepared.result.value}`)
    }
  }

  await new Promise((resolve) => setTimeout(resolve, shot.wait ?? 400))

  // Pomiar jest pod nadpisaniem widoku, żeby mierzył to, co naprawdę będzie na zdjęciu.
  if (shot.measure) {
    const measured = await send('Runtime.evaluate', {
      expression: shot.measure,
      awaitPromise: true,
      returnByValue: true,
    })
    console.log(`[${shot.name}] pomiar: ${measured.result?.value}`)
  }

  if (shot.capture === false) continue

  let clip = shot.clip
  if (shot.clipTo) {
    const found = await send('Runtime.evaluate', {
      expression: shot.clipTo,
      awaitPromise: true,
      returnByValue: true,
    })
    const rect = found.result?.value
    if (!rect) {
      console.error(`[${shot.name}] clipTo nie znalazło elementu - robię cały widok`)
    } else {
      clip = {
        x: rect.x - (shot.clipPad ?? 0),
        y: rect.y - (shot.clipPad ?? 0),
        width: rect.width + 2 * (shot.clipPad ?? 0),
        height: rect.height + 2 * (shot.clipPad ?? 0),
        scale: shot.clipScale ?? 1,
      }
    }
  }
  if (clip && clip.scale === undefined) clip.scale = shot.clipScale ?? 1

  const { data } = await send('Page.captureScreenshot', {
    format: 'png',
    fromSurface: true,
    captureBeyondViewport: false,
    ...(clip ? { clip } : {}),
  })

  const buffer = Buffer.from(data, 'base64')
  const path = join(outDir, `${shot.name}.png`)
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, buffer)

  const size = pngSize(buffer)
  const rozmiar = size.png ? `${size.width}x${size.height}` : 'NIE-PNG'
  console.log(
    `[${shot.name}] widok ${width}x${height} @${scale}x -> ${path} (${rozmiar}, ${(buffer.length / 1024).toFixed(0)} kB)`,
  )
}

await send('Emulation.clearDeviceMetricsOverride')
socket.close()
