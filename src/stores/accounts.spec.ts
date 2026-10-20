import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { backend } from '../api/backend'
import type { Account, OpenTab } from '../api/contract'
import { useAccountsStore } from './accounts'
import { useSelectionStore } from './selection'

// Backend podmieniamy w całości: te testy sprawdzają logikę store'u, a nie Tauri.
vi.mock('../api/backend', () => ({
  backend: {
    listAccounts: vi.fn(),
    registerAccounts: vi.fn(),
    saveAccounts: vi.fn(),
    addAccount: vi.fn(),
    removeAccount: vi.fn(),
    getOpenTabs: vi.fn(),
    saveOpenTabs: vi.fn(),
    startConnection: vi.fn(),
    stopConnection: vi.fn(),
    restartConnection: vi.fn(),
  },
  events: {},
}))

function account(id: string, servers: string[]): Account {
  return {
    id,
    label: id,
    type: 'microsoft',
    login: `${id}@example.com`,
    servers: servers.map((address) => ({ address, version: null })),
    tabOrder: 0,
  }
}

const SAVED_ACCOUNTS = [account('acc-1', ['play.example.com']), account('acc-2', ['mc.example.net'])]

const SAVED_TABS: OpenTab[] = [
  { accountId: 'acc-2', server: 'mc.example.net' },
  { accountId: 'acc-1', server: 'play.example.com' },
]

/** Lista kont z **ostatniego** wywołania `save_accounts` (to ona idzie do pliku). */
function lastSaved(): Account[] {
  const calls = vi.mocked(backend.saveAccounts).mock.calls
  return calls[calls.length - 1]?.[0] ?? []
}

/** Konto nowe, tworzone w UI (`tabOrder` i `servers` nadaje store). */
function newAccount(id: string): Omit<Account, 'tabOrder' | 'servers'> {
  return { id, label: id, type: 'offline', login: id }
}

describe('store kont - sesja zakładek', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.listAccounts).mockResolvedValue(SAVED_ACCOUNTS)
    vi.mocked(backend.registerAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.saveAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.addAccount).mockResolvedValue(undefined)
    vi.mocked(backend.removeAccount).mockResolvedValue(undefined)
    vi.mocked(backend.getOpenTabs).mockResolvedValue(SAVED_TABS)
    vi.mocked(backend.saveOpenTabs).mockImplementation(async (tabs) => tabs)
    vi.mocked(backend.startConnection).mockResolvedValue(undefined)
    vi.mocked(backend.stopConnection).mockResolvedValue(undefined)
    vi.mocked(backend.restartConnection).mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  /** Stan po starcie aplikacji z włączonym przełącznikiem przywracania. */
  async function restoreWithTabs() {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()
    await accounts.restoreOpenTabs(true)
    return accounts
  }

  it('przywraca okna zakładek, ale nie uruchamia żadnego procesu', async () => {
    const accounts = await restoreWithTabs()

    expect(accounts.connections.map((conn) => conn.id)).toEqual([
      'acc-2::mc.example.net',
      'acc-1::play.example.com',
    ])
    expect(accounts.activeConnectionId).toBe('acc-2::mc.example.net')
    // Najważniejsza reguła: przywrócenie zakładek to nie połączenie.
    expect(backend.startConnection).not.toHaveBeenCalled()
    expect(accounts.statusOf('acc-1::play.example.com')).toBe('stopped')
    expect(accounts.statusOf('acc-2::mc.example.net')).toBe('stopped')
  })

  it('wyłączony przełącznik nie pyta o zakładki i nic nie otwiera', async () => {
    const accounts = useAccountsStore()
    await accounts.restoreOpenTabs(false)

    expect(backend.getOpenTabs).not.toHaveBeenCalled()
    expect(accounts.connections).toEqual([])
    expect(accounts.activeConnectionId).toBeNull()
  })

  it('pomija zakładki kont, których nie ma w panelu', async () => {
    const accounts = useAccountsStore()
    accounts.accounts = [account('acc-1', ['play.example.com'])]
    await accounts.restoreOpenTabs(true)

    expect(accounts.connections.map((conn) => conn.id)).toEqual(['acc-1::play.example.com'])
    expect(accounts.activeConnectionId).toBe('acc-1::play.example.com')
  })

  it('zapisuje zakładkę w sesji po jej otwarciu', async () => {
    const accounts = useAccountsStore()
    accounts.accounts = SAVED_ACCOUNTS
    vi.mocked(backend.getOpenTabs).mockResolvedValue([])

    await accounts.openConnection('acc-1', 'play.example.com')

    expect(backend.startConnection).toHaveBeenCalledWith('acc-1', 'play.example.com')
    // Zapis sesji jest „w tle”, więc czekamy na mikro-zadania.
    await Promise.resolve()
    await Promise.resolve()
    expect(backend.saveOpenTabs).toHaveBeenCalledWith([
      { accountId: 'acc-1', server: 'play.example.com' },
    ])
  })

  it('zamyka zakładkę dopiero po zatrzymaniu procesu (R-18)', async () => {
    const accounts = await restoreWithTabs()
    let releaseStop: (() => void) | undefined
    vi.mocked(backend.stopConnection).mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          releaseStop = resolve
        }),
    )

    const closing = accounts.closeConnection('acc-2::mc.example.net')
    expect(releaseStop).toBeDefined()
    // Stop jeszcze leci, więc zakładka musi zostać widoczna - inaczej proces
    // zostałby bez okna i bez sposobu, żeby go zatrzymać.
    expect(accounts.connections.map((conn) => conn.id)).toContain('acc-2::mc.example.net')

    releaseStop?.()
    await closing

    expect(accounts.connections.map((conn) => conn.id)).toEqual(['acc-1::play.example.com'])
  })

  it('nieudany stop zostawia zakładkę i pokazuje błąd (R-18)', async () => {
    const accounts = await restoreWithTabs()
    vi.mocked(backend.stopConnection).mockRejectedValue({
      code: 'connection.notRunning',
      params: {},
    })

    await accounts.closeConnection('acc-2::mc.example.net')

    expect(accounts.connections.map((conn) => conn.id)).toContain('acc-2::mc.example.net')
    expect(accounts.errors[0]?.code).toBe('connection.notRunning')
    // Sesja nie zmienia się, bo nic się nie zamknęło.
    expect(backend.saveOpenTabs).not.toHaveBeenCalled()
  })

  it('zamknięcie zakładki usuwa stan zaznaczenia jej wierszy (R-41)', async () => {
    const accounts = await restoreWithTabs()
    const selections = useSelectionStore()
    const id = 'acc-2::mc.example.net'
    selections.selectRow(id, 1)
    expect(selections.has(id)).toBe(true)

    await accounts.closeConnection(id)

    // Pusty wpis w mapie też jest wyciekiem - stan ma zniknąć, a nie tylko wyczyścić się.
    expect(selections.has(id)).toBe(false)
    expect(selections.sizeFor(id)).toBe(0)
  })

  it('zamknięcie zakładki usuwa ją z sesji', async () => {
    const accounts = await restoreWithTabs()
    expect(backend.saveOpenTabs).not.toHaveBeenCalled() // przywracanie samo nic nie zapisuje

    await accounts.closeConnection('acc-2::mc.example.net')
    await Promise.resolve()
    await Promise.resolve()

    expect(backend.stopConnection).toHaveBeenCalledWith('acc-2::mc.example.net')
    expect(backend.saveOpenTabs).toHaveBeenLastCalledWith([
      { accountId: 'acc-1', server: 'play.example.com' },
    ])
  })
})

describe('store kont - przełącznik dotyczy zakładek, nie kont', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.listAccounts).mockResolvedValue(SAVED_ACCOUNTS)
    vi.mocked(backend.registerAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.saveAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.addAccount).mockResolvedValue(undefined)
    vi.mocked(backend.removeAccount).mockResolvedValue(undefined)
    vi.mocked(backend.saveOpenTabs).mockImplementation(async (tabs) => tabs)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  /**
   * Regresja: gdy przełącznik był wyłączony, panel
   * startował **pusty**, a pierwszy zapis podmieniał `accounts.json` samym nowym
   * kontem. Dziś przełącznik dotyczy wyłącznie zakładek - konta wracają do panelu
   * zawsze, więc zapis nie ma czego zgubić.
   */
  it('wyłączony przełącznik zabiera zakładki, a konta zostawia w panelu', async () => {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()
    await accounts.restoreOpenTabs(false)

    expect(accounts.accounts.map((item) => item.id)).toEqual(['acc-1', 'acc-2'])
    expect(accounts.connections).toEqual([])
    expect(backend.getOpenTabs).not.toHaveBeenCalled()
  })

  it('dodanie konta zapisuje je razem z wcześniejszymi', async () => {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()

    await accounts.addAccount(newAccount('acc-3'))

    expect(lastSaved().map((item) => item.id)).toEqual(['acc-1', 'acc-2', 'acc-3'])
    expect(lastSaved()[2].tabOrder).toBe(1)
  })

  /**
   * S-05: `save_accounts` może odmówić zapisu (np. plik w kwarantannie).
   * Wtedy konto nie jest dodane - store cofa też rejestrację w menedżerze.
   */
  it('nieudany zapis cofa dodanie konta (S-05)', async () => {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()
    vi.mocked(backend.saveAccounts).mockRejectedValue({
      code: 'store.fileQuarantined',
      params: { file: 'accounts.json' },
    })

    const added = await accounts.addAccount(newAccount('acc-3'))

    expect(added).toBe(false)
    expect(accounts.accounts.map((item) => item.id)).toEqual(['acc-1', 'acc-2'])
    expect(backend.removeAccount).toHaveBeenCalledWith('acc-3')
    expect(accounts.errors[0]?.code).toBe('store.fileQuarantined')
  })

  it('usunięcie konta usuwa tylko to jedno', async () => {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()

    await accounts.removeAccount('acc-1')

    expect(lastSaved().map((item) => item.id)).toEqual(['acc-2'])
  })

  it('wczytane konta są rejestrowane, ale plik nie jest nadpisywany', async () => {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()

    expect(backend.registerAccounts).toHaveBeenCalledWith(SAVED_ACCOUNTS)
    expect(backend.saveAccounts).not.toHaveBeenCalled()
    expect(accounts.accounts.map((item) => item.id)).toEqual(['acc-1', 'acc-2'])
  })

  /**
   * R-01: zapis odrzuca całą listę, gdy choć jeden wpis jest niepoprawny.
   * Wcześniej start wołał `save_accounts`, więc taki plik zostawiał **pusty
   * panel**, a pierwszy `persist` nadpisywał `accounts.json` bez starych kont.
   */
  it('niepoprawny wpis nie czyści panelu ani pliku', async () => {
    const broken: Account = { ...account('bad/id', []), login: 'a\nb' }
    vi.mocked(backend.listAccounts).mockResolvedValue([...SAVED_ACCOUNTS, broken])

    const accounts = useAccountsStore()
    await accounts.restoreAccounts()

    expect(accounts.accounts.map((item) => item.id)).toEqual(['acc-1', 'acc-2', 'bad/id'])
    expect(backend.saveAccounts).not.toHaveBeenCalled()
  })
})

describe('store kont - autoprzewijanie per zakładka', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(backend.listAccounts).mockResolvedValue(SAVED_ACCOUNTS)
    vi.mocked(backend.registerAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.saveAccounts).mockResolvedValue(undefined)
    vi.mocked(backend.getOpenTabs).mockResolvedValue(SAVED_TABS)
    vi.mocked(backend.startConnection).mockResolvedValue(undefined)
    vi.mocked(backend.stopConnection).mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  /** Dwie przywrócone zakładki - po jednej na konto. */
  async function twoTabs() {
    const accounts = useAccountsStore()
    await accounts.restoreAccounts()
    await accounts.restoreOpenTabs(true)
    return accounts
  }

  it('nowe i przywrócone zakładki śledzą domyślnie', async () => {
    const accounts = await twoTabs()

    expect(accounts.isFollowing('acc-1::play.example.com')).toBe(true)
    expect(accounts.isFollowing('acc-2::mc.example.net')).toBe(true)
  })

  it('pauza w jednej zakładce nie wyłącza śledzenia w drugiej', async () => {
    const accounts = await twoTabs()

    accounts.setFollowing('acc-1::play.example.com', false)

    expect(accounts.isFollowing('acc-1::play.example.com')).toBe(false)
    expect(accounts.isFollowing('acc-2::mc.example.net')).toBe(true)
  })

  it('zamknięcie zakładki kasuje jej stan śledzenia', async () => {
    const accounts = await twoTabs()
    accounts.setFollowing('acc-1::play.example.com', false)

    await accounts.closeConnection('acc-1::play.example.com')

    expect(accounts.isFollowing('acc-1::play.example.com')).toBe(true)
  })

  it('ponowne uruchomienie zakładki wznawia śledzenie', async () => {
    const accounts = await twoTabs()
    accounts.setFollowing('acc-1::play.example.com', false)

    await accounts.startConnection('acc-1::play.example.com')

    expect(accounts.isFollowing('acc-1::play.example.com')).toBe(true)
    expect(backend.startConnection).toHaveBeenCalledWith('acc-1', 'play.example.com')
  })
})

describe('store kont - karty błędów (R-39)', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('ten sam błąd nie mnoży kart', () => {
    const accounts = useAccountsStore()
    accounts.pushError({ code: 'connection.notRunning', params: { accountId: 'acc-1' } })
    accounts.pushError({ code: 'connection.notRunning', params: { accountId: 'acc-1' } })

    expect(accounts.errors).toHaveLength(1)
  })

  it('ten sam kod z innymi parametrami to inna karta', () => {
    const accounts = useAccountsStore()
    accounts.pushError({ code: 'process.startFailed', params: { accountId: 'acc-1' } })
    accounts.pushError({ code: 'process.startFailed', params: { accountId: 'acc-2' } })

    expect(accounts.errors).toHaveLength(2)
  })

  it('ponad pięć błędów zostawia pięć najnowszych', () => {
    const accounts = useAccountsStore()
    for (let index = 0; index < 20; index += 1) {
      accounts.pushError({ code: 'process.startFailed', params: { accountId: `acc-${index}` } })
    }

    expect(accounts.errors).toHaveLength(5)
    // Wypadają najstarsze - zostaje ogon listy, w kolejności napływu.
    expect(accounts.errors.map((error) => error.params.accountId)).toEqual([
      'acc-15',
      'acc-16',
      'acc-17',
      'acc-18',
      'acc-19',
    ])
  })

  it('ukrycie karty nie blokuje pokazania tego samego błędu ponownie', () => {
    const accounts = useAccountsStore()
    accounts.pushError({ code: 'connection.notRunning', params: {} })
    accounts.dismissError(accounts.errors[0].id)
    accounts.pushError({ code: 'connection.notRunning', params: {} })

    expect(accounts.errors).toHaveLength(1)
  })
})
