/**
 * @jest-environment jsdom
 */
import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import React from 'react'
import { CopyButton } from '../copy-button'

describe('CopyButton', () => {
  beforeEach(() => {
    jest.useFakeTimers()
    Object.assign(navigator, {
      clipboard: { writeText: jest.fn().mockResolvedValue(undefined) },
    })
  })

  afterEach(() => {
    jest.useRealTimers()
    jest.restoreAllMocks()
  })

  it('renders with Copy aria-label initially', () => {
    render(<CopyButton text="abc" />)
    expect(screen.getByRole('button', { name: 'Copy to clipboard' })).toBeInTheDocument()
  })

  it('shows Copied! aria-label after click', async () => {
    render(<CopyButton text="abc" />)
    await act(async () => {
      await userEvent.click(screen.getByRole('button'))
    })
    expect(screen.getByRole('button', { name: 'Copied!' })).toBeInTheDocument()
  })

  it('calls clipboard.writeText with the provided text', async () => {
    render(<CopyButton text="my-text" />)
    await act(async () => {
      await userEvent.click(screen.getByRole('button'))
    })
    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('my-text')
  })

  it('resets aria-label back to Copy to clipboard after resetMs', async () => {
    render(<CopyButton text="abc" resetMs={500} />)
    await act(async () => {
      await userEvent.click(screen.getByRole('button'))
    })
    expect(screen.getByRole('button', { name: 'Copied!' })).toBeInTheDocument()

    act(() => {
      jest.advanceTimersByTime(500)
    })
    expect(screen.getByRole('button', { name: 'Copy to clipboard' })).toBeInTheDocument()
  })
})
