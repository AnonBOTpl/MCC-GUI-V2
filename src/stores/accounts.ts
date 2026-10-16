// Store kont i połączeń.
//
// Model „konto + lista serwerów": konto to tożsamość logowania (jeden katalog =
// jedna sesja MCC), a każde **połączenie** (konto + serwer) to osobny proces i
// osobna zakładka. Wszystkie mapy (statusy, logi, prompty logowania) są
// kluczowane **identyfikatorem połączenia**, bo Rust wysyła go w polu `accountId`
// zdarzeń (kontrakt Tauri).
//
// Trwałość (Etap 4): po każdej zmianie listy kont zapisujemy ją przez
// `save_accounts`, a przy starcie wczytujemy zapisane konta (`restoreAccounts`).
// **Konta wracają zawsze** - przełącznik „Przywracaj zakładki przy starcie”
// dotyczy wyłącznie zakładek (połączeń), nigdy listy kont w panelu po lewej.
//
// Sesja: zestaw otwartych zakładek zapisujemy przy każdej zmianie
// (`persistOpenTabs`), a przy starcie odtwarzamy same **okna**
// (`restoreOpenTabs`). Gdy przełącznik jest wyłączony, ta funkcja nic nie robi
// i aplikacja startuje bez zakładek. Procesów MCC nie uruchamiamy nigdy sami -
// zakładki wracają jako `stopped`, a startuje je użytkownik.

import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { backend, events } from '../api/backend'
import {
  connectionId,
  toConnection,
  type Account,
  type AccountStatus,
  type Connection,
  type DeviceCodePayload,
  type ErrorPayload,
  type NewAccount,
  type ServerEntry,
} from '../api/contract'
import { diagCounters } from '../log/diag'
import { connectionsToTabs, tabsToConnections } from '../session/tabs'
import { useLogsStore } from './logs'

/** Zdarzenie błędu z czasem odbioru - do wyświetlenia w UI i auto-ukrycia. */
export interface AppErrorEvent extends ErrorPayload {
  id: number
  ts: number
}

export const useAccountsStore = defineStore('accounts', () => {
  const logs = useLogsStore()

  /** Zapisane konta (z listami serwerów). */
  const accounts = ref<Account[]>([])
  /** Otwarte połączenia = zakładki. */
  const connections = ref<Connection[]>([])
  /** Status per identyfikator połączenia. */
  const statuses = ref<Record<string, AccountStatus>>({})
  /**
   * Autoprzewijanie logu **per zakładka** (`true` = widok jedzie za nowymi
   * liniami). Brak wpisu znaczy `true`, więc nowe i przywrócone zakładki śledzą
   * od razu, a pauza w jednej zakładce nie wyłącza śledzenia w drugiej.
   */
  const following = ref<Record<string, boolean>>({})
  const activeConnectionId = ref<string | null>(null)
  const errors = ref<AppErrorEvent[]>([])
  /** Prompt logowania Microsoft per połączenie (Etap 3). */
  const loginPrompts = ref<Record<string, { url: string; code: string }>>({})
  /** Czy wysyłka trafia do wszystkich połączeń (przełącznik „Send to all"). */
  const sendToAll = ref(false)

  let errorSeq = 0

  const activeConnection = computed(
    () => connections.value.find((conn) => conn.id === activeConnectionId.value) ?? null,
  )

  const activeAccount = computed(() => {
    const conn = activeConnection.value
    if (!conn) return null
    return accounts.value.find((account) => account.id === conn.accountId) ?? null
  })

  const activeStatus = computed<AccountStatus>(() =>
    activeConnectionId.value ? statuses.value[activeConnectionId.value] ?? 'stopped' : 'stopped',
  )

  /** Liczba połączeń w stanie uznawanym za aktywne (uruchomione lub w drodze). */
  const activeCount = computed(
    () =>
      connections.value.filter((conn) => {
        const status = statuses.value[conn.id] ?? 'stopped'
        return status !== 'stopped' && status !== 'crashed'
      }).length,
  )

  function statusOf(id: string): AccountStatus {
    return statuses.value[id] ?? 'stopped'
  }

  /** Czy zakładka tego połączenia śledzi nowe linie (domyślnie tak). */
  function isFollowing(id: string): boolean {
    return following.value[id] ?? true
  }

  /** Włącza albo wyłącza autoprzewijanie w **jednej** zakładce. */
  function setFollowing(id: string, value: boolean): void {
    following.value = { ...following.value, [id]: value }
  }

  /** Zbiorczy status konta dla panelu: najpierw działające, potem reszta. */
  function accountStatus(accountId: string): AccountStatus {
    const list = connections.value
      .filter((conn) => conn.accountId === accountId)
      .map((conn) => statusOf(conn.id))
    const order: AccountStatus[] = [
      'running',
      'awaiting_login',
      'starting',
      'stopping',
      'crashed',
      'stopped',
    ]
    for (const candidate of order) {
      if (list.includes(candidate)) return candidate
    }
    return 'stopped'
  }

  function isOpen(accountId: string, server: string): boolean {
    return connections.value.some((conn) => conn.id === connectionId(accountId, server))
  }

  function loginPromptFor(id: string): { url: string; code: string } | undefined {
    return loginPrompts.value[id]
  }

  /** Kopiuje kod logowania do schowka (przez Rusta - patrz `backend.copyLoginCode`). */
  async function copyLoginCode(code: string): Promise<boolean> {
    try {
      await backend.copyLoginCode(code)
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
  }

  /** Otwiera adres logowania w przeglądarce (walidacja adresu jest w Rustcie). */
  async function openLoginUrl(url: string): Promise<void> {
    try {
      await backend.openLoginUrl(url)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  function pushError(payload: ErrorPayload): void {
    errorSeq += 1
    errors.value = [...errors.value, { ...payload, id: errorSeq, ts: Date.now() }]
  }

  function dismissError(id: number): void {
    errors.value = errors.value.filter((error) => error.id !== id)
  }

  /** Kolejny numer zakładki - konta nie zmieniają kolejności same z siebie. */
  function nextTabOrder(): number {
    return accounts.value.reduce((max, account) => Math.max(max, account.tabOrder + 1), 0)
  }

  function setActiveConnection(id: string): void {
    activeConnectionId.value = id
    logs.markRead(id)
  }

  /** Usuwa ślad połączenia z pamięci (bez wołania Rusta). */
  function dropConnectionLocally(id: string): void {
    const index = connections.value.findIndex((conn) => conn.id === id)
    if (index !== -1) {
      const remaining = connections.value.filter((conn) => conn.id !== id)
      connections.value = remaining
      if (activeConnectionId.value === id) {
        activeConnectionId.value = remaining[Math.min(index, remaining.length - 1)]?.id ?? null
      }
    }
    logs.remove(id)
    const nextStatuses = { ...statuses.value }
    delete nextStatuses[id]
    statuses.value = nextStatuses
    const nextPrompts = { ...loginPrompts.value }
    delete nextPrompts[id]
    loginPrompts.value = nextPrompts
    const nextFollowing = { ...following.value }
    delete nextFollowing[id]
    following.value = nextFollowing
  }

  /**
   * Zapisuje bieżącą listę kont na dysk. Błąd tylko raportujemy (nie blokuje UI).
   *
   * `accounts` to **zawsze pełna lista z pliku plus zmiany z panelu**: przy starcie
   * wczytujemy zapisane konta bez względu na przełącznik zakładek, więc zapis nie
   * może zgubić niczego, czego użytkownik nie widzi.
   */
  async function persist(): Promise<void> {
    try {
      await backend.saveAccounts(accounts.value)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /**
   * Wczytuje zapisane konta przy starcie. Konta wracają **zawsze** - przełącznik
   * „Przywracaj zakładki przy starcie” decyduje tylko o zakładkach
   * (`restoreOpenTabs`), nigdy o panelu kont. Połączeń **nie otwieramy**.
   */
  async function restoreAccounts(): Promise<void> {
    try {
      const saved = await backend.listAccounts()
      if (saved.length === 0) return
      // Rust rejestruje konta (tworzy ich katalogi) bez uruchamiania procesów.
      await backend.saveAccounts(saved)
      accounts.value = [...saved].sort((a, b) => a.tabOrder - b.tabOrder)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /**
   * Przywraca okna połączeń z poprzedniej sesji (`session.json`).
   *
   * **Nie uruchamia żadnego procesu**: zakładki wracają jako `stopped`, więc MCC
   * nie łączy się z serwerem bez decyzji użytkownika. Pierwsza przywrócona
   * zakładka staje się aktywna, żeby użytkownik od razu widział, co wróciło.
   */
  async function restoreOpenTabs(enabled: boolean): Promise<void> {
    if (!enabled) return
    try {
      const saved = await backend.getOpenTabs()
      if (saved.length === 0) return
      const restored = tabsToConnections(saved, accounts.value)
      if (restored.length === 0) return
      connections.value = restored
      activeConnectionId.value = restored[0].id
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /**
   * Zapisuje zestaw otwartych zakładek, żeby wrócił po restarcie aplikacji.
   * Wołane przy każdej zmianie listy zakładek (otwarcie, zamknięcie, usunięcie
   * konta albo serwera); błąd tylko raportujemy - nie blokuje UI.
   */
  async function persistOpenTabs(): Promise<void> {
    try {
      await backend.saveOpenTabs(connectionsToTabs(connections.value))
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /** Dodaje konto (bez uruchamiania czegokolwiek). */
  async function addAccount(input: NewAccount): Promise<boolean> {
    const account: Account = { ...input, servers: [], tabOrder: nextTabOrder() }
    accounts.value = [...accounts.value, account]
    try {
      await backend.addAccount(account)
      await persist()
      return true
    } catch (error) {
      accounts.value = accounts.value.filter((item) => item.id !== account.id)
      pushError(error as ErrorPayload)
      return false
    }
  }

  /** Usuwa konto razem z jego otwartymi połączeniami. */
  async function removeAccount(id: string): Promise<void> {
    for (const conn of connections.value.filter((conn) => conn.accountId === id)) {
      dropConnectionLocally(conn.id)
    }
    accounts.value = accounts.value.filter((account) => account.id !== id)
    void persistOpenTabs()
    try {
      await backend.removeAccount(id)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
    await persist()
  }

  /** Wstawia serwer do listy, a dla istniejącego adresu aktualizuje wersję. */
  function upsertServer(
    servers: ServerEntry[],
    address: string,
    version: string | null,
  ): ServerEntry[] {
    const normalized = version !== null && version.trim() !== '' ? version.trim() : null
    const lowered = address.toLowerCase()
    const index = servers.findIndex((entry) => entry.address.toLowerCase() === lowered)
    if (index === -1) return [...servers, { address, version: normalized }]
    return servers.map((entry, position) =>
      position === index ? { ...entry, address, version: normalized } : entry,
    )
  }

  /**
   * Dopisuje serwer do konta. Gdy `connect` jest prawdziwe, od razu otwiera
   * połączenie (startuje proces) - to wariant „Add & connect" z panelu.
   * Ten sam adres to aktualizacja wersji, nie duplikat.
   */
  async function addServer(
    accountId: string,
    server: string,
    version: string | null,
    connect: boolean,
  ): Promise<boolean> {
    const trimmed = server.trim()
    if (trimmed === '') return false

    try {
      await backend.addServer(accountId, trimmed, version)
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }

    accounts.value = accounts.value.map((account) =>
      account.id === accountId
        ? { ...account, servers: upsertServer(account.servers, trimmed, version) }
        : account,
    )
    await persist()

    if (connect) await openConnection(accountId, trimmed)
    return true
  }

  /** Usuwa serwer z konta (Rust zatrzymuje jego proces). */
  async function removeServer(accountId: string, server: string): Promise<void> {
    const conn = toConnection(accountId, server)
    const wasOpen = connections.value.some((item) => item.id === conn.id)
    try {
      await backend.removeServer(accountId, server)
    } catch (error) {
      pushError(error as ErrorPayload)
      return
    }
    if (wasOpen) dropConnectionLocally(conn.id)
    if (wasOpen) void persistOpenTabs()
    const lowered = server.toLowerCase()
    accounts.value = accounts.value.map((account) =>
      account.id === accountId
        ? {
            ...account,
            servers: account.servers.filter(
              (item) => item.address.toLowerCase() !== lowered,
            ),
          }
        : account,
    )
    await persist()
  }

  /**
   * Otwiera zakładkę połączenia i startuje jego proces. Gdy zakładka już jest,
   * tylko ją aktywuje, a zatrzymane połączenie dodatkowo uruchamia.
   */
  async function openConnection(accountId: string, server: string): Promise<void> {
    const conn = toConnection(accountId, server)
    const existing = connections.value.find((item) => item.id === conn.id)
    if (existing) {
      setActiveConnection(conn.id)
      const status = statusOf(conn.id)
      if (status === 'stopped' || status === 'crashed') await startConnection(conn.id)
      return
    }

    connections.value = [...connections.value, conn]
    statuses.value = { ...statuses.value, [conn.id]: 'starting' }
    setActiveConnection(conn.id)
    void persistOpenTabs()

    try {
      await backend.startConnection(accountId, server)
    } catch (error) {
      pushError(error as ErrorPayload)
      dropConnectionLocally(conn.id)
    }
  }

  /** Zatrzymuje proces, ale zostawia zakładkę (można ją znowu uruchomić). */
  async function disconnectConnection(id: string): Promise<void> {
    try {
      await backend.stopConnection(id)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /** Zamyka zakładkę połączenia i zatrzymuje proces (serwer zostaje zapisany). */
  async function closeConnection(id: string): Promise<void> {
    const conn = connections.value.find((item) => item.id === id)
    if (!conn) return
    dropConnectionLocally(id)
    void persistOpenTabs()
    try {
      await backend.stopConnection(id)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /** Startuje zatrzymane połączenie (przycisk „Uruchom" w nagłówku). */
  async function startConnection(id: string): Promise<void> {
    const conn = connections.value.find((item) => item.id === id)
    if (!conn) return
    logs.clear(id)
    // Log startuje od zera, więc wracamy do śledzenia - inaczej po ponownym
    // uruchomieniu zakładka z wyłączonym autoprzewijaniem stałaby zamrożona.
    setFollowing(id, true)
    const nextPrompts = { ...loginPrompts.value }
    delete nextPrompts[id]
    loginPrompts.value = nextPrompts
    statuses.value = { ...statuses.value, [id]: 'starting' }
    try {
      await backend.startConnection(conn.accountId, conn.server)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /** Ponownie uruchamia połączenie. */
  async function restartConnection(id: string): Promise<void> {
    const conn = connections.value.find((item) => item.id === id)
    if (!conn) return
    logs.clear(id)
    // Tak jak przy starcie: świeży log = znowu śledzimy.
    setFollowing(id, true)
    const nextPrompts = { ...loginPrompts.value }
    delete nextPrompts[id]
    loginPrompts.value = nextPrompts
    statuses.value = { ...statuses.value, [id]: 'starting' }
    try {
      await backend.restartConnection(conn.accountId, conn.server)
    } catch (error) {
      pushError(error as ErrorPayload)
    }
  }

  /**
   * Wysyła wpis do jednego połączenia. Używane przez pole wpisywania i przez
   * presety szybkich akcji, żeby wysyłka miała jedną wspólną ścieżkę.
   */
  async function sendToConnection(id: string, text: string): Promise<boolean> {
    try {
      await backend.sendInput(id, text)
      return true
    } catch (error) {
      pushError(error as ErrorPayload)
      return false
    }
  }

  /**
   * Wysyła wpis do wszystkich działających połączeń i zwraca liczbę tych,
   * do których naprawdę dotarł (Rust pomija zatrzymane połączenia).
   */
  async function sendToAllRunning(text: string): Promise<number> {
    try {
      const delivered = await backend.sendInputAll(text)
      return delivered.length
    } catch (error) {
      pushError(error as ErrorPayload)
      return 0
    }
  }

  /**
   * Wysyła wpis do aktywnego połączenia albo do wszystkich (przełącznik).
   * Zwraca liczbę połączeń, do których wpis naprawdę dotarł - dzięki temu
   * szybkie akcje wiedzą, kiedy nie było do kogo wysłać.
   */
  async function send(text: string): Promise<number> {
    const trimmed = text.trim()
    if (trimmed === '') return 0

    if (sendToAll.value) return sendToAllRunning(trimmed)
    if (!activeConnectionId.value) return 0
    return (await sendToConnection(activeConnectionId.value, trimmed)) ? 1 : 0
  }

  /** Podłącza zdarzenia z Rusta. Zwraca funkcję odsubskrybowania. */
  function subscribe(): () => void {
    const unlisteners: Array<() => void> = []

    void events
      .onLog((payload) => {
        // Diagnostyka: licznik odebranych zdarzeń (patrz `src/log/diag.ts`).
        diagCounters.received += 1
        logs.append(
          payload.accountId,
          [{ stream: payload.stream, raw: payload.line, ts: payload.ts }],
          activeConnectionId.value,
        )
      })
      .then((unlisten) => unlisteners.push(unlisten))

    void events
      .onStatus((payload) => {
        statuses.value = { ...statuses.value, [payload.accountId]: payload.status }
      })
      .then((unlisten) => unlisteners.push(unlisten))

    void events
      .onError((payload) => {
        pushError(payload)
      })
      .then((unlisten) => unlisteners.push(unlisten))

    void events
      .onDeviceCode((payload: DeviceCodePayload) => {
        loginPrompts.value = {
          ...loginPrompts.value,
          [payload.accountId]: { url: payload.url, code: payload.code },
        }
      })
      .then((unlisten) => unlisteners.push(unlisten))

    void events
      .onJoined((payload) => {
        // Po połączeniu prompt jest już nieaktualny.
        const nextPrompts = { ...loginPrompts.value }
        delete nextPrompts[payload.accountId]
        loginPrompts.value = nextPrompts
      })
      .then((unlisten) => unlisteners.push(unlisten))

    return () => {
      for (const unlisten of unlisteners) unlisten()
    }
  }

  return {
    accounts,
    connections,
    statuses,
    activeConnectionId,
    activeConnection,
    activeAccount,
    activeStatus,
    activeCount,
    errors,
    sendToAll,
    loginPrompts,
    statusOf,
    isFollowing,
    setFollowing,
    accountStatus,
    isOpen,
    loginPromptFor,
    copyLoginCode,
    openLoginUrl,
    setActiveConnection,
    restoreAccounts,
    restoreOpenTabs,
    persistOpenTabs,
    addAccount,
    removeAccount,
    addServer,
    removeServer,
    openConnection,
    disconnectConnection,
    closeConnection,
    startConnection,
    restartConnection,
    send,
    sendToConnection,
    sendToAllRunning,
    pushError,
    dismissError,
    subscribe,
  }
})
