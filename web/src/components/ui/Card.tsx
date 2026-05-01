import React from 'react'

interface CardProps {
  children: React.ReactNode
  className?: string
  title?: string
}

export default function Card({ children, className = '', title }: CardProps) {
  return (
    <div
      className={`rounded-xl border border-slate-700 bg-slate-800/50 p-6 shadow-lg ${className}`}
    >
      {title && (
        <h3 className="mb-4 text-sm font-medium text-emerald-300">{title}</h3>
      )}
      {children}
    </div>
  )
}
