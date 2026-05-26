/**
 * @jest-environment jsdom
 */
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import React from 'react'
import { Pagination } from '../pagination'

const baseProps = {
  hasMore: true,
  onNext: jest.fn(),
  onPrev: jest.fn(),
  pageSize: 10 as const,
  onPageSizeChange: jest.fn(),
}

describe('Pagination', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  it('renders Previous and Load more buttons', () => {
    render(<Pagination {...baseProps} />)
    expect(screen.getByRole('button', { name: 'Previous page' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Next page' })).toBeInTheDocument()
  })

  it('disables Previous when page=1', () => {
    render(<Pagination {...baseProps} page={1} />)
    expect(screen.getByRole('button', { name: 'Previous page' })).toBeDisabled()
  })

  it('enables Previous when page>1', () => {
    render(<Pagination {...baseProps} page={2} />)
    expect(screen.getByRole('button', { name: 'Previous page' })).not.toBeDisabled()
  })

  it('disables Previous when no nextCursor and page is undefined', () => {
    render(<Pagination {...baseProps} nextCursor={undefined} page={undefined} />)
    expect(screen.getByRole('button', { name: 'Previous page' })).toBeDisabled()
  })

  it('enables Previous when nextCursor is set and page is undefined', () => {
    render(<Pagination {...baseProps} nextCursor="cursor123" page={undefined} />)
    expect(screen.getByRole('button', { name: 'Previous page' })).not.toBeDisabled()
  })

  it('disables Load more when hasMore=false', () => {
    render(<Pagination {...baseProps} hasMore={false} />)
    expect(screen.getByRole('button', { name: 'Next page' })).toBeDisabled()
  })

  it('enables Load more when hasMore=true', () => {
    render(<Pagination {...baseProps} hasMore={true} />)
    expect(screen.getByRole('button', { name: 'Next page' })).not.toBeDisabled()
  })

  it('calls onNext when Load more is clicked', async () => {
    render(<Pagination {...baseProps} />)
    await userEvent.click(screen.getByRole('button', { name: 'Next page' }))
    expect(baseProps.onNext).toHaveBeenCalledTimes(1)
  })

  it('calls onPrev when Previous is clicked', async () => {
    render(<Pagination {...baseProps} page={2} />)
    await userEvent.click(screen.getByRole('button', { name: 'Previous page' }))
    expect(baseProps.onPrev).toHaveBeenCalledTimes(1)
  })

  it('calls onPageSizeChange with selected value', async () => {
    render(<Pagination {...baseProps} />)
    await userEvent.selectOptions(screen.getByRole('combobox'), '25')
    expect(baseProps.onPageSizeChange).toHaveBeenCalledWith(25)
  })

  it('shows page indicator when page prop is provided', () => {
    render(<Pagination {...baseProps} page={3} />)
    expect(screen.getByText('Page 3')).toBeInTheDocument()
  })

  it('does not show page indicator when page is omitted', () => {
    render(<Pagination {...baseProps} />)
    expect(screen.queryByText(/^Page/)).not.toBeInTheDocument()
  })
})
