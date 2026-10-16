#!/usr/bin/env node
// Sterowanie i inspekcja prawdziwego okna aplikacji Tauri przez Chrome DevTools
// Protocol (CDP). Ocenia wyrażenie JS w działającym webview i wypisuje wynik jako
// JSON. Dzięki temu można czytać DOM, klikać przyciski i sprawdzać, co się
// wyrenderowało, bez człowieka przy klawiaturze.
//
//   echo 'document.body.innerText' | node scripts/ui-drive.mjs
//
// Wymaga włączonego portu debug, który jest domyślnie WYŁĄCZONY ze względów
// bezpieczeństwa. Dodaj tymczasowo do okna w `src-tauri/tauri.conf.json`:
//
//   "additionalBrowserArgs": "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --remote-debugging-port=9222 --remote-allow-origins=*"
//
// potem `npm run tauri dev` i steruj aplikacją z terminala. Usunięcie tej linii
// wystarczy, żeby port znowu był zamknięty.

const ENDPOINT = process.env.CDP_ENDPOINT ?? 'http://127.0.0.1:9222'

const expression = await new Promise((resolve) => {
  let data = ''
  process.stdin.setEncoding('utf8')
  process.stdin.on('data', (chunk) => {
    data += chunk
  })
  process.stdin.on('end', () => resolve(data))
})

if (!expression.trim()) {
  console.error('Podaj wyrażenie JS na stdin.')
  process.exit(2)
}

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

const reply = await new Promise((resolve) => {
  socket.onmessage = (event) => {
    const message = JSON.parse(event.data)
    if (message.id === 1) resolve(message)
  }
  socket.send(
    JSON.stringify({
      id: 1,
      method: 'Runtime.evaluate',
      params: {
        expression,
        awaitPromise: true,
        returnByValue: true,
        userGesture: true,
      },
    }),
  )
})

socket.close()

if (reply.result?.exceptionDetails) {
  const details = reply.result.exceptionDetails
  console.error(`WYJĄTEK: ${details.text} ${details.exception?.description ?? ''}`)
  process.exit(1)
}

const value = reply.result?.result?.value
console.log(typeof value === 'string' ? value : JSON.stringify(value, null, 2))
