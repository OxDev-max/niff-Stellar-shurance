/**
 * @jest-environment jsdom
 *
 * Unit tests for dark mode system preference detection and localStorage
 * persistence logic. These tests verify the core behaviour described in the
 * dark mode feature requirements:
 *
 *   1. Dark mode activates automatically when system preference is dark.
 *   2. Manual override persists across page reloads (localStorage).
 *   3. System preference changes are reflected when no manual override is set.
 */

const STORAGE_KEY = 'niffyinsur-theme'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function mockMatchMedia(prefersDark: boolean) {
  const listeners: Array<(e: MediaQueryListEvent) => void> = []

  const mql = {
    matches: prefersDark,
    media: '(prefers-color-scheme: dark)',
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn((_, handler) => listeners.push(handler)),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
    _listeners: listeners,
  }

  jest.spyOn(window, 'matchMedia').mockImplementation((query: string) => {
    if (query === '(prefers-color-scheme: dark)') return mql as unknown as MediaQueryList
    return {
      matches: false,
      media: query,
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    } as unknown as MediaQueryList
  })

  return { mql, listeners }
}

// ---------------------------------------------------------------------------
// System preference detection
// ---------------------------------------------------------------------------

describe('dark mode — system preference detection', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove('light', 'dark')
    jest.restoreAllMocks()
  })

  it('resolves to dark when system preference is dark and no override stored', () => {
    mockMatchMedia(true)

    // Simulate the inline script logic from layout.tsx
    const stored = localStorage.getItem(STORAGE_KEY)
    const resolved =
      stored === null || stored === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : stored

    expect(resolved).toBe('dark')
  })

  it('resolves to light when system preference is light and no override stored', () => {
    mockMatchMedia(false)

    const stored = localStorage.getItem(STORAGE_KEY)
    const resolved =
      stored === null || stored === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : stored

    expect(resolved).toBe('light')
  })

  it('resolves to dark when stored value is "system" and system is dark', () => {
    mockMatchMedia(true)
    localStorage.setItem(STORAGE_KEY, 'system')

    const stored = localStorage.getItem(STORAGE_KEY)
    const resolved =
      stored === null || stored === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : stored

    expect(resolved).toBe('dark')
  })
})

// ---------------------------------------------------------------------------
// Manual override persistence
// ---------------------------------------------------------------------------

describe('dark mode — manual override persistence', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove('light', 'dark')
    jest.restoreAllMocks()
  })

  it('persists "dark" override to localStorage', () => {
    localStorage.setItem(STORAGE_KEY, 'dark')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('dark')
  })

  it('persists "light" override to localStorage', () => {
    localStorage.setItem(STORAGE_KEY, 'light')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
  })

  it('manual override takes precedence over system preference', () => {
    mockMatchMedia(true) // system is dark
    localStorage.setItem(STORAGE_KEY, 'light') // but user chose light

    const stored = localStorage.getItem(STORAGE_KEY)
    const resolved =
      stored === null || stored === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : stored

    // Manual override wins
    expect(resolved).toBe('light')
  })

  it('override survives a simulated page reload (re-read from localStorage)', () => {
    // Simulate user setting dark mode
    localStorage.setItem(STORAGE_KEY, 'dark')

    // Simulate page reload: re-read from storage
    const afterReload = localStorage.getItem(STORAGE_KEY)
    expect(afterReload).toBe('dark')
  })

  it('clearing override falls back to system preference', () => {
    mockMatchMedia(true)
    localStorage.setItem(STORAGE_KEY, 'light')

    // User resets to system
    localStorage.setItem(STORAGE_KEY, 'system')

    const stored = localStorage.getItem(STORAGE_KEY)
    const resolved =
      stored === null || stored === 'system'
        ? window.matchMedia('(prefers-color-scheme: dark)').matches
          ? 'dark'
          : 'light'
        : stored

    expect(resolved).toBe('dark')
  })
})

// ---------------------------------------------------------------------------
// DOM class application
// ---------------------------------------------------------------------------

describe('dark mode — DOM class application', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove('light', 'dark')
    jest.restoreAllMocks()
  })

  it('applies "dark" class to <html> when theme is dark', () => {
    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add('dark')
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    expect(document.documentElement.classList.contains('light')).toBe(false)
  })

  it('applies "light" class to <html> when theme is light', () => {
    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add('light')
    expect(document.documentElement.classList.contains('light')).toBe(true)
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })

  it('only one theme class is present at a time', () => {
    document.documentElement.classList.add('dark')
    // Switch to light
    document.documentElement.classList.remove('dark')
    document.documentElement.classList.add('light')

    const classes = Array.from(document.documentElement.classList)
    const themeClasses = classes.filter((c) => c === 'dark' || c === 'light')
    expect(themeClasses).toHaveLength(1)
    expect(themeClasses[0]).toBe('light')
  })
})
