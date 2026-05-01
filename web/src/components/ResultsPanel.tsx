import { useState, useRef } from 'react'
import { useProductContext } from '../context/ProductContext.tsx'
import Card from './ui/Card.tsx'
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts'

function formatNumber(n: number): string {
  if (!Number.isFinite(n)) return '—'
  return n.toLocaleString('en-US', { maximumFractionDigits: 6 })
}

const CURVE_COLORS: Record<string, string> = {
  delta: '#60a5fa',
  gamma: '#f472b6',
  theta: '#fbbf24',
  vega: '#a78bfa',
  rho: '#fb923c',
  vanna: '#a3e635',
  charm: '#f87171',
  vomma: '#818cf8',
  speed: '#fbbf24',
  price: '#34d399',
}

export default function ResultsPanel() {
  const { results, activeAnalytics, error, loading } = useProductContext()
  const [activeMetrics, setActiveMetrics] = useState<Record<string, boolean>>({})
  const lastDataKeysRef = useRef<string>('')

  if (loading && !results) {
    return (
      <Card>
        <div className="text-sm text-slate-400">Calculating...</div>
      </Card>
    )
  }

  if (error) {
    return (
      <Card>
        <div className="rounded-lg border border-red-800 bg-red-900/30 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      </Card>
    )
  }

  if (!results) return null

  // Price result
  if (activeAnalytics === 'price' && 'price' in results) {
    const price = results.price as string
    const currency = (results.currency as string) || 'USD'
    return (
      <Card title="Price Result">
        <div className="flex items-baseline gap-2">
          <span className="text-3xl font-bold text-white">
            {formatNumber(parseFloat(price))}
          </span>
          <span className="text-sm text-emerald-400">{currency}</span>
        </div>
      </Card>
    )
  }

  // Greeks result
  if (activeAnalytics === 'greeks') {
    const entries = Object.entries(results).filter(
      ([k]) => k !== 'currency'
    ) as [string, number][]
    return (
      <Card title="Greeks">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3">
          {entries.map(([name, value]) => (
            <div
              key={name}
              className="rounded-lg border border-emerald-800/50 bg-emerald-950/30 px-4 py-3"
            >
              <div className="text-xs capitalize text-emerald-400">
                {name}
              </div>
              <div className="mt-1 text-lg font-semibold text-white">
                {formatNumber(value)}
              </div>
            </div>
          ))}
        </div>
      </Card>
    )
  }

  // Second-order Greeks result
  if (activeAnalytics === 'second-order-greeks') {
    const entries = Object.entries(results) as [string, number][]
    return (
      <Card title="Higher-Order Greeks">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {entries.map(([name, value]) => (
            <div
              key={name}
              className="rounded-lg border border-emerald-800/50 bg-emerald-950/30 px-4 py-3"
            >
              <div className="text-xs capitalize text-emerald-400">
                {name}
              </div>
              <div className="mt-1 text-lg font-semibold text-white">
                {formatNumber(value)}
              </div>
            </div>
          ))}
        </div>
      </Card>
    )
  }

  // Curve results
  if (activeAnalytics === 'curve' && 'points' in results) {
    const points = (results.points as Array<Record<string, unknown>>) || []
    if (points.length === 0) {
      return (
        <Card title="Curve">
          <div className="text-sm text-slate-400">No data points returned.</div>
        </Card>
      )
    }

    // Validate first point has expected shape
    const firstPoint = points[0]
    if (!firstPoint || typeof firstPoint !== 'object') {
      return (
        <Card title="Curve">
          <div className="text-sm text-slate-400">Invalid curve data.</div>
        </Card>
      )
    }

    // Build chart data safely
    const chartData: Array<Record<string, number>> = []
    for (const p of points) {
      if (!p || typeof p !== 'object') continue
      const xVal = p.x
      const xNum =
        typeof xVal === 'number'
          ? xVal
          : typeof xVal === 'string'
            ? parseFloat(xVal)
            : NaN
      if (!Number.isFinite(xNum)) continue

      const row: Record<string, number> = { x: xNum }
      for (const [k, v] of Object.entries(p)) {
        if (k === 'x') continue
        const num =
          typeof v === 'number'
            ? v
            : typeof v === 'string'
              ? parseFloat(v)
              : NaN
        if (Number.isFinite(num)) {
          row[k] = num
        }
      }
      chartData.push(row)
    }

    if (chartData.length === 0) {
      return (
        <Card title="Curve">
          <div className="text-sm text-slate-400">
            Could not parse curve data.
          </div>
        </Card>
      )
    }

    const dataKeys = Object.keys(chartData[0]).filter((k) => k !== 'x')
    const dataKeysSig = dataKeys.sort().join(',')

    // Initialise active metrics only when data keys change
    if (lastDataKeysRef.current !== dataKeysSig) {
      lastDataKeysRef.current = dataKeysSig
      const initial: Record<string, boolean> = {}
      dataKeys.forEach((k) => (initial[k] = true))
      // Defer state update to avoid render-phase side effects
      setTimeout(() => setActiveMetrics(initial), 0)
    }

    const tooltipStyle = {
      backgroundColor: '#1e293b',
      border: '1px solid #334155',
      borderRadius: '8px',
      color: '#e2e8f0',
    }

    const toggleMetric = (key: string) => {
      setActiveMetrics((prev) => ({ ...prev, [key]: !prev[key] }))
    }

    return (
      <Card title="Curve">
        {/* Metric toggles */}
        <div className="mb-4 flex flex-wrap gap-3">
          {dataKeys.map((key) => (
            <label
              key={key}
              className="flex cursor-pointer items-center gap-1.5 text-xs text-slate-300"
            >
              <input
                type="checkbox"
                checked={!!activeMetrics[key]}
                onChange={() => toggleMetric(key)}
                className="h-3.5 w-3.5 rounded border-slate-600 bg-slate-800"
              />
              <span
                className="inline-block h-2 w-2 rounded-full"
                style={{
                  backgroundColor: CURVE_COLORS[key] || '#94a3b8',
                }}
              />
              <span className="capitalize">{key}</span>
            </label>
          ))}
        </div>

        <div style={{ width: '100%', minHeight: 400 }}>
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={chartData} margin={{ top: 5, right: 20, left: 10, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis
                dataKey="x"
                tick={{ fill: '#94a3b8', fontSize: 12 }}
                stroke="#475569"
                label={{ value: 'X', position: 'insideBottom', offset: -2, fill: '#94a3b8' }}
              />
              <YAxis
                tick={{ fill: '#94a3b8', fontSize: 12 }}
                stroke="#475569"
              />
              <Tooltip
                contentStyle={tooltipStyle}
                formatter={(v: unknown) =>
                  formatNumber(
                    typeof v === 'number' ? v : parseFloat(String(v))
                  )
                }
              />
              <Legend wrapperStyle={{ color: '#94a3b8' }} />
              {dataKeys.map((key) => (
                <Line
                  key={key}
                  type="monotone"
                  dataKey={key}
                  stroke={CURVE_COLORS[key] || '#94a3b8'}
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4 }}
                  hide={!activeMetrics[key]}
                />
              ))}
            </LineChart>
          </ResponsiveContainer>
        </div>
      </Card>
    )
  }

  // Generic fallback
  const entries = Object.entries(results) as [string, unknown][]
  return (
    <Card title="Result">
      <div className="space-y-2">
        {entries.map(([key, value]) => (
          <div key={key} className="flex justify-between text-sm">
            <span className="capitalize text-slate-400">{key}</span>
            <span className="text-white">{String(value)}</span>
          </div>
        ))}
      </div>
    </Card>
  )
}
