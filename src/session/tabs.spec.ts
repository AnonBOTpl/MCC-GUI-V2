import { describe, expect, it } from 'vitest'
import type { Account, Connection, OpenTab } from '../api/contract'
import { connectionsToTabs, tabsToConnections } from './tabs'

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

function tab(accountId: string, server: string): OpenTab {
  return { accountId, server }
}

const ACCOUNTS: Account[] = [
  account('acc-1', ['play.example.com', 'mc.example.net']),
  account('acc-2', ['play.example.com']),
]

describe('sesja zakładek', () => {
  it('przywraca zakładki istniejących kont i serwerów w kolejności zapisu', () => {
    const restored = tabsToConnections(
      [tab('acc-2', 'play.example.com'), tab('acc-1', 'mc.example.net'), tab('acc-1', 'play.example.com')],
      ACCOUNTS,
    )

    expect(restored.map((connection) => connection.id)).toEqual([
      'acc-2::play.example.com',
      'acc-1::mc.example.net',
      'acc-1::play.example.com',
    ])
  })

  it('pomija zakładki usuniętego konta i skasowanego serwera', () => {
    const restored = tabsToConnections(
      [tab('acc-1', 'play.example.com'), tab('ghost', 'play.example.com'), tab('acc-1', 'gone.net')],
      ACCOUNTS,
    )

    expect(restored).toHaveLength(1)
    expect(restored[0].id).toBe('acc-1::play.example.com')
  })

  it('dopasowuje adres bez rozróżniania wielkości liter i bierze go z konta', () => {
    const restored = tabsToConnections([tab('acc-1', '  Play.Example.COM ')], ACCOUNTS)

    expect(restored).toHaveLength(1)
    expect(restored[0].server).toBe('play.example.com')
  })

  it('usuwa duplikaty zakładek', () => {
    const restored = tabsToConnections(
      [tab('acc-1', 'play.example.com'), tab('acc-1', 'PLAY.EXAMPLE.COM')],
      ACCOUNTS,
    )

    expect(restored).toHaveLength(1)
  })

  it('nie tworzy niczego, gdy nie ma kont (wyłączone przywracanie)', () => {
    expect(tabsToConnections([tab('acc-1', 'play.example.com')], [])).toEqual([])
  })

  it('zapisuje sesję w kształcie, który rozumie Rust', () => {
    const connections: Connection[] = [
      { id: 'acc-1::play.example.com', accountId: 'acc-1', server: 'play.example.com' },
    ]

    expect(connectionsToTabs(connections)).toEqual([
      { accountId: 'acc-1', server: 'play.example.com' },
    ])
  })
})
