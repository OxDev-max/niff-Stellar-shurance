'use client'

import { Check, Copy } from 'lucide-react'
import * as React from 'react'
import { useCopyToClipboard } from '@/hooks/use-copy-to-clipboard'
import { cn } from '@/lib/utils'

export interface CopyButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'onClick'> {
  text: string
  /** Milliseconds before the checkmark resets. Defaults to 2000. */
  resetMs?: number
}

export function CopyButton({ text, resetMs, className, ...props }: CopyButtonProps) {
  const { copy, copied } = useCopyToClipboard(resetMs)

  return (
    <button
      type="button"
      onClick={() => copy(text)}
      aria-label={copied ? 'Copied!' : 'Copy to clipboard'}
      className={cn(
        'inline-flex items-center justify-center rounded p-1 text-muted-foreground transition-colors hover:text-foreground focus:outline-none focus:ring-2 focus:ring-ring min-h-[44px] min-w-[44px]',
        className,
      )}
      {...props}
    >
      {copied ? (
        <Check className="h-4 w-4 text-green-600" aria-hidden="true" />
      ) : (
        <Copy className="h-4 w-4" aria-hidden="true" />
      )}
      <span className="sr-only" aria-live="polite">
        {copied ? 'Copied!' : ''}
      </span>
    </button>
  )
}
