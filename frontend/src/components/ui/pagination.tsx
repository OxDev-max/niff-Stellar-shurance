'use client'

import * as React from 'react'
import { cn } from '@/lib/utils'

const PAGE_SIZE_OPTIONS = [10, 25, 50] as const
export type PageSize = (typeof PAGE_SIZE_OPTIONS)[number]

export interface PaginationProps {
  /** Cursor token for the next page; undefined when on the first page. */
  nextCursor?: string
  /** Whether more pages exist after the current one. */
  hasMore: boolean
  onNext: () => void
  onPrev: () => void
  pageSize: PageSize
  onPageSizeChange: (size: PageSize) => void
  /** 1-based current page number for display. */
  page?: number
  className?: string
}

export function Pagination({
  nextCursor,
  hasMore,
  onNext,
  onPrev,
  pageSize,
  onPageSizeChange,
  page,
  className,
}: PaginationProps) {
  const hasPrev = page !== undefined ? page > 1 : nextCursor !== undefined

  return (
    <nav
      aria-label="Pagination"
      className={cn('flex flex-wrap items-center justify-between gap-4', className)}
    >
      <div className="flex items-center gap-2">
        <label htmlFor="page-size-select" className="text-sm text-muted-foreground">
          Rows per page
        </label>
        <select
          id="page-size-select"
          value={pageSize}
          onChange={(e) => onPageSizeChange(Number(e.target.value) as PageSize)}
          className="min-h-[44px] rounded border border-input bg-background px-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
        >
          {PAGE_SIZE_OPTIONS.map((size) => (
            <option key={size} value={size}>
              {size}
            </option>
          ))}
        </select>
      </div>

      <div className="flex items-center gap-3">
        {page !== undefined && (
          <span
            aria-live="polite"
            aria-current="page"
            className="text-sm text-muted-foreground"
          >
            Page {page}
          </span>
        )}

        <button
          type="button"
          onClick={onPrev}
          disabled={!hasPrev}
          aria-label="Previous page"
          className="min-h-[44px] min-w-[44px] rounded border border-input bg-background px-4 text-sm font-medium hover:bg-accent disabled:cursor-not-allowed disabled:opacity-40 focus:outline-none focus:ring-2 focus:ring-ring"
        >
          Previous
        </button>

        <button
          type="button"
          onClick={onNext}
          disabled={!hasMore}
          aria-label="Next page"
          className="min-h-[44px] min-w-[44px] rounded border border-input bg-background px-4 text-sm font-medium hover:bg-accent disabled:cursor-not-allowed disabled:opacity-40 focus:outline-none focus:ring-2 focus:ring-ring"
        >
          Load more
        </button>
      </div>
    </nav>
  )
}
