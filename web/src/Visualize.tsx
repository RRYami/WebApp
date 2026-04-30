import { useState, useCallback } from 'react'
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

interface VisualizeProps {
  spot: number
  riskFreeRate: number
  volatility: number
  timeToMaturity: number
  dividendYield: number
  instrument: string
  currentStrike: string
}

interface PricePoint {
  x: number
  'Call Price'?: number
  'Put Price'?: number
}

interface GreeksPoint {
  x: number
  'Call-Delta'?: number
  'Call-Gamma'?: number
  'Call-Theta'?: number
  'Call-Vega'?: number
  'Call-Rho'?: number
  'Call-Phi'?: number
  'Put-Delta'?: number
  'Put-Gamma'?: number
  'Put-Theta'?: number
  'Put-Vega'?: number
  'Put-Rho'?: number
  'Put-Phi'?: number
}

const colors = {
  delta: '#60a5fa',
  gamma: '#f472b6',
  theta: '#fbbf24',
  vega: '#a78bfa',
  rho: '#fb923c',
  phi: '#22d3ee',
}

const tooltipStyle = {
  backgroundColor: '#1e293b',
  border: '1px solid #334155',
  borderRadius: '8px',
  color: '#e2e8f0',
}

function formatAxis(value: number): string {
  if (Math.abs(value) >= 1000) return (value / 1000).toFixed(1) + 'k'
  if (Math.abs(value) >= 1) return value.toFixed(2)
  return value.toFixed(4)
}

export default function Visualize({
  spot,
  riskFreeRate,
  volatility,
  timeToMaturity,
  dividendYield,
  instrument,
  currentStrike,
}: VisualizeProps) {
  const defaultMin = Math.round(spot * 0.7)
  const defaultMax = Math.round(spot * 1.3)

  const [minX, setMinX] = useState(defaultMin.toString())
  const [maxX, setMaxX] = useState(defaultMax.toString())
  const [steps, setSteps] = useState('30')
  const [fixedStrike, setFixedStrike] = useState(currentStrike)
  const [showCall, setShowCall] = useState(true)
  const [showPut, setShowPut] = useState(true)
  const [activeGreeks, setActiveGreeks] = useState({
    delta: true,
    gamma: true,
    theta: true,
    vega: true,
    rho: false,
    phi: false,
  })
  const [priceData, setPriceData] = useState<PricePoint[]>([])
  const [greeksData, setGreeksData] = useState<GreeksPoint[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const generateRange = useCallback((): number[] => {
    const min = parseFloat(minX)
    const max = parseFloat(maxX)
    const n = parseInt(steps)
    const step = (max - min) / (n - 1)
    return Array.from({ length: n }, (_, i) => min + step * i)
  }, [minX, maxX, steps])

  const fetchPriceCurve = async (optionType: string): Promise<PricePoint[]> => {
    const strikes = generateRange()
    const res = await fetch('/api/price/curve', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        instrument: instrument.toLowerCase(),
        option_type: optionType,
        spot: spot.toString(),
        risk_free_rate: (riskFreeRate / 100).toString(),
        volatility: (volatility / 100).toString(),
        time_to_maturity: timeToMaturity,
        dividend_yield: instrument === 'American' ? (dividendYield / 100).toString() : undefined,
        strikes: strikes.map((s) => s.toString()),
      }),
    })
    if (!res.ok) {
      const err = await res.json()
      throw new Error(err.error || `Price curve HTTP ${res.status}`)
    }
    const json = await res.json()
    return json.points.map((p: any) => ({
      x: parseFloat(p.strike),
      [`${optionType} Price`]: parseFloat(p.price),
    }))
  }

  const fetchGreeksCurve = async (optionType: string): Promise<GreeksPoint[]> => {
    const spots = generateRange()
    const res = await fetch('/api/greeks/curve', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        instrument: instrument.toLowerCase(),
        option_type: optionType,
        spot: spot.toString(),
        risk_free_rate: (riskFreeRate / 100).toString(),
        volatility: (volatility / 100).toString(),
        time_to_maturity: timeToMaturity,
        dividend_yield: instrument === 'American' ? (dividendYield / 100).toString() : undefined,
        strikes: [fixedStrike],
        spots: spots.map((s) => s.toString()),
        fixed_strike: fixedStrike,
      }),
    })
    if (!res.ok) {
      const err = await res.json()
      throw new Error(err.error || `Greeks curve HTTP ${res.status}`)
    }
    const json = await res.json()
    return json.points.map((p: any) => ({
      x: parseFloat(p.strike),
      [`${optionType}-Delta`]: p.delta,
      [`${optionType}-Gamma`]: p.gamma,
      [`${optionType}-Theta`]: p.theta,
      [`${optionType}-Vega`]: p.vega,
      [`${optionType}-Rho`]: p.rho,
      [`${optionType}-Phi`]: p.phi,
    }))
  }

  const handleGenerate = async () => {
    setLoading(true)
    setError(null)
    try {
      // Fetch price curves
      const pricePromises: Promise<PricePoint[]>[] = []
      if (showCall) pricePromises.push(fetchPriceCurve('Call'))
      if (showPut) pricePromises.push(fetchPriceCurve('Put'))
      const priceResults = await Promise.all(pricePromises)

      // Merge price data by x value
      const priceMap = new Map<number, PricePoint>()
      priceResults.flat().forEach((p) => {
        const existing = priceMap.get(p.x) || { x: p.x }
        priceMap.set(p.x, { ...existing, ...p })
      })
      setPriceData(Array.from(priceMap.values()).sort((a, b) => a.x - b.x))

      // Fetch greeks curves
      const greeksPromises: Promise<GreeksPoint[]>[] = []
      if (showCall) greeksPromises.push(fetchGreeksCurve('Call'))
      if (showPut) greeksPromises.push(fetchGreeksCurve('Put'))
      const greeksResults = await Promise.all(greeksPromises)

      // Merge greeks data by x value
      const greeksMap = new Map<number, GreeksPoint>()
      greeksResults.flat().forEach((p) => {
        const existing = greeksMap.get(p.x) || { x: p.x }
        greeksMap.set(p.x, { ...existing, ...p })
      })
      setGreeksData(Array.from(greeksMap.values()).sort((a, b) => a.x - b.x))
    } catch (e: any) {
      setError(e.message || 'Failed to generate curves')
    } finally {
      setLoading(false)
    }
  }

  const hasPriceData = priceData.length > 0
  const hasGreeksData = greeksData.length > 0

  const inputClass =
    'w-full rounded-lg border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500'

  const labelClass = 'mb-1 block text-xs font-medium text-slate-300'

  const greekKeys = ['delta', 'gamma', 'theta', 'vega', 'rho', 'phi'] as const

  return (
    <div className="space-y-6">
      {/* Controls */}
      <div className="rounded-xl border border-slate-700 bg-slate-800/50 p-6 shadow-lg">
        <div className="mb-4 grid grid-cols-2 gap-4">
          <div>
            <label className={labelClass}>Min {hasGreeksData ? 'Spot' : 'Strike'}</label>
            <input
              type="number"
              value={minX}
              onChange={(e) => setMinX(e.target.value)}
              className={inputClass}
            />
          </div>
          <div>
            <label className={labelClass}>Max {hasGreeksData ? 'Spot' : 'Strike'}</label>
            <input
              type="number"
              value={maxX}
              onChange={(e) => setMaxX(e.target.value)}
              className={inputClass}
            />
          </div>
        </div>

        <div className="mb-4 grid grid-cols-2 gap-4">
          <div>
            <label className={labelClass}>Steps</label>
            <input
              type="number"
              min="2"
              max="100"
              value={steps}
              onChange={(e) => setSteps(e.target.value)}
              className={inputClass}
            />
          </div>
          <div>
            <label className={labelClass}>Fixed Strike (for Greeks)</label>
            <input
              type="number"
              step="0.01"
              value={fixedStrike}
              onChange={(e) => setFixedStrike(e.target.value)}
              className={inputClass}
            />
          </div>
        </div>

        <div className="mb-4 flex gap-4">
          <label className="flex items-center gap-2 text-sm text-slate-300">
            <input
              type="checkbox"
              checked={showCall}
              onChange={(e) => setShowCall(e.target.checked)}
              className="h-4 w-4 rounded border-slate-600 bg-slate-800 text-blue-600"
            />
            Call
          </label>
          <label className="flex items-center gap-2 text-sm text-slate-300">
            <input
              type="checkbox"
              checked={showPut}
              onChange={(e) => setShowPut(e.target.checked)}
              className="h-4 w-4 rounded border-slate-600 bg-slate-800 text-blue-600"
            />
            Put
          </label>
        </div>

        <button
          onClick={handleGenerate}
          disabled={loading || (!showCall && !showPut)}
          className="w-full rounded-lg bg-blue-600 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {loading ? 'Generating...' : 'Generate Curves'}
        </button>

        {error && (
          <div className="mt-3 rounded-lg border border-red-800 bg-red-900/30 px-3 py-2 text-sm text-red-300">
            {error}
          </div>
        )}
      </div>

      {/* Price Chart */}
      {hasPriceData && (
        <div className="rounded-xl border border-slate-700 bg-slate-800/50 p-6 shadow-lg">
          <h3 className="mb-4 text-sm font-semibold text-emerald-300">
            Option Price vs Strike
          </h3>
          <ResponsiveContainer width="100%" height={350}>
            <LineChart
              data={priceData as any[]}
              margin={{ top: 5, right: 20, left: 10, bottom: 5 }}
            >
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis
                dataKey="x"
                tick={{ fill: '#94a3b8', fontSize: 12 }}
                tickFormatter={formatAxis}
                stroke="#475569"
                label={{ value: 'Strike', position: 'insideBottom', offset: -2, fill: '#94a3b8' }}
              />
              <YAxis
                tick={{ fill: '#94a3b8', fontSize: 12 }}
                tickFormatter={formatAxis}
                stroke="#475569"
                label={{ value: 'Price', angle: -90, position: 'insideLeft', fill: '#94a3b8' }}
              />
              <Tooltip contentStyle={tooltipStyle} formatter={(v: any) => Number(v)?.toFixed(4)} />
              <Legend wrapperStyle={{ color: '#94a3b8' }} />
              {showCall && (
                <Line
                  type="monotone"
                  dataKey="Call Price"
                  stroke="#34d399"
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4 }}
                />
              )}
              {showPut && (
                <Line
                  type="monotone"
                  dataKey="Put Price"
                  stroke="#f43f5e"
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4 }}
                />
              )}
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Greeks Chart */}
      {hasGreeksData && (
        <div className="rounded-xl border border-slate-700 bg-slate-800/50 p-6 shadow-lg">
          <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
            <h3 className="text-sm font-semibold text-emerald-300">
              Greeks vs Underlying Price
            </h3>
            <div className="flex flex-wrap gap-3">
              {greekKeys.map((key) => (
                <label key={key} className="flex items-center gap-1.5 text-xs text-slate-300">
                  <input
                    type="checkbox"
                    checked={activeGreeks[key]}
                    onChange={(e) =>
                      setActiveGreeks((prev) => ({ ...prev, [key]: e.target.checked }))
                    }
                    className="h-3.5 w-3.5 rounded border-slate-600 bg-slate-800"
                  />
                  <span
                    className="inline-block h-2 w-2 rounded-full"
                    style={{ backgroundColor: colors[key] }}
                  />
                  {key.charAt(0).toUpperCase() + key.slice(1)}
                </label>
              ))}
            </div>
          </div>
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={greeksData as any[]} margin={{ top: 5, right: 20, left: 10, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
              <XAxis
                dataKey="x"
                tick={{ fill: '#94a3b8', fontSize: 12 }}
                tickFormatter={formatAxis}
                stroke="#475569"
                label={{
                  value: 'Underlying Price (Spot)',
                  position: 'insideBottom',
                  offset: -2,
                  fill: '#94a3b8',
                }}
              />
              <YAxis
                tick={{ fill: '#94a3b8', fontSize: 12 }}
                tickFormatter={formatAxis}
                stroke="#475569"
              />
              <Tooltip contentStyle={tooltipStyle} formatter={(v: any) => Number(v)?.toFixed(6)} />
              <Legend wrapperStyle={{ color: '#94a3b8' }} />
              {greekKeys.map((greek) =>
                activeGreeks[greek] ? (
                  <>
                    {showCall && (
                      <Line
                        key={`Call-${greek}`}
                        type="monotone"
                        dataKey={`Call-${greek.charAt(0).toUpperCase() + greek.slice(1)}`}
                        stroke={colors[greek]}
                        strokeWidth={2}
                        dot={false}
                        activeDot={{ r: 4 }}
                      />
                    )}
                    {showPut && (
                      <Line
                        key={`Put-${greek}`}
                        type="monotone"
                        dataKey={`Put-${greek.charAt(0).toUpperCase() + greek.slice(1)}`}
                        stroke={colors[greek]}
                        strokeWidth={2}
                        strokeDasharray="6 4"
                        dot={false}
                        activeDot={{ r: 4 }}
                      />
                    )}
                  </>
                ) : null
              )}
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}
    </div>
  )
}