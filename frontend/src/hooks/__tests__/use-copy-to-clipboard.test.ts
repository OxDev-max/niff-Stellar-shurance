/**
 * @jest-environment jsdom
 */
import { act, renderHook } from '@testing-library/react'
import { useCopyToClipboard } from '../use-copy-to-clipboard'

describe('useCopyToClipboard', () => {
  beforeEach(() => {
    jest.useFakeTimers()
  })

  afterEach(() => {
    jest.useRealTimers()
    jest.restoreAllMocks()
  })

  it('returns copied=false initially', () => {
    const { result } = renderHook(() => useCopyToClipboard())
    expect(result.current.copied).toBe(false)
  })

  it('sets copied=true after successful copy', async () => {
    Object.assign(navigator, {
      clipboard: { writeText: jest.fn().mockResolvedValue(undefined) },
    })

    const { result } = renderHook(() => useCopyToClipboard())
    await act(async () => {
      await result.current.copy('hello')
    })

    expect(result.current.copied).toBe(true)
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('hello')
  })

  it('resets copied to false after resetMs', async () => {
    Object.assign(navigator, {
      clipboard: { writeText: jest.fn().mockResolvedValue(undefined) },
    })

    const { result } = renderHook(() => useCopyToClipboard(1000))
    await act(async () => {
      await result.current.copy('hello')
    })
    expect(result.current.copied).toBe(true)

    act(() => {
      jest.advanceTimersByTime(1000)
    })
    expect(result.current.copied).toBe(false)
  })

  it('leaves copied=false when copy fails', async () => {
    Object.assign(navigator, {
      clipboard: { writeText: jest.fn().mockRejectedValue(new Error('denied')) },
    })

    const { result } = renderHook(() => useCopyToClipboard())
    await act(async () => {
      await result.current.copy('hello')
    })

    expect(result.current.copied).toBe(false)
  })

  it('uses execCommand fallback when clipboard API is unavailable', async () => {
    // Remove clipboard API
    const originalClipboard = navigator.clipboard
    Object.defineProperty(navigator, 'clipboard', { value: undefined, configurable: true })

    const execCommandSpy = jest.spyOn(document, 'execCommand').mockReturnValue(true)

    const { result } = renderHook(() => useCopyToClipboard())
    await act(async () => {
      await result.current.copy('fallback text')
    })

    expect(execCommandSpy).toHaveBeenCalledWith('copy')
    expect(result.current.copied).toBe(true)

    // Restore
    Object.defineProperty(navigator, 'clipboard', { value: originalClipboard, configurable: true })
  })
})
