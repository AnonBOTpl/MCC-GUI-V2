// Sesja: które zakładki (połączenia) były otwarte przy zamknięciu aplikacji.
//
// Moduł jest **czysty** (żadnego Tauri ani Pinii), więc reguły przywracania da się
// przetestować bez aplikacji - tak jak `src/log/*` i `src/presets/*`.
//
// Najważniejsza reguła: przywracamy **okna**, a nie połączenia. Zakładka wraca w
// stanie `stopped`, bez procesu MCC, i użytkownik sam ją uruchamia.

import { toConnection, type Account, type Connection, type OpenTab } from '../api/contract'

/**
 * Zamienia zakładki z `session.json` na połączenia gotowe do pokazania w UI.
 *
 * Pomija wpisy, dla których nie ma już konta albo serwera na tym koncie (konto
 * mogło zostać usunięte, serwer skasowany), usuwa duplikaty i zachowuje kolejność
 * zapisu - to kolejność zakładek. Adresy porównujemy bez rozróżniania wielkości
 * liter, a do wyniku wchodzi adres **z konta**, żeby identyfikator połączenia był
 * dokładnie ten sam, którego używa reszta aplikacji.
 */
export function tabsToConnections(
  tabs: readonly OpenTab[],
  accounts: readonly Account[],
): Connection[] {
  const connections: Connection[] = []
  const seen = new Set<string>()

  for (const tab of tabs) {
    const account = accounts.find((item) => item.id === tab.accountId)
    if (!account) continue

    const wanted = tab.server.trim().toLowerCase()
    const server = account.servers.find((entry) => entry.address.toLowerCase() === wanted)
    if (!server) continue

    const connection = toConnection(account.id, server.address)
    if (seen.has(connection.id)) continue
    seen.add(connection.id)
    connections.push(connection)
  }

  return connections
}

/** Zamienia otwarte połączenia na kształt zapisywany w sesji. */
export function connectionsToTabs(connections: readonly Connection[]): OpenTab[] {
  return connections.map((connection) => ({
    accountId: connection.accountId,
    server: connection.server,
  }))
}
