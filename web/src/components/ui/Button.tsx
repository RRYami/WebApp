import React from 'react'

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'danger'
  loading?: boolean
}

export default function Button({
  children,
  variant = 'primary',
  loading = false,
  disabled,
  className = '',
  ...props
}: ButtonProps) {
  const base =
    'rounded-lg px-4 py-2.5 text-sm font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900'

  const variants = {
    primary:
      'bg-blue-600 text-white hover:bg-blue-500 focus:ring-blue-500 disabled:bg-blue-800 disabled:cursor-not-allowed disabled:opacity-50',
    secondary:
      'bg-slate-700 text-slate-200 hover:bg-slate-600 focus:ring-slate-500 disabled:bg-slate-800 disabled:cursor-not-allowed disabled:opacity-50',
    danger:
      'bg-red-600 text-white hover:bg-red-500 focus:ring-red-500 disabled:bg-red-800 disabled:cursor-not-allowed disabled:opacity-50',
  }

  return (
    <button
      disabled={disabled || loading}
      className={`${base} ${variants[variant]} ${className}`}
      {...props}
    >
      {loading ? 'Loading...' : children}
    </button>
  )
}
