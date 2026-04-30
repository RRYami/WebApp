import { useState } from 'react'
import Visualize from './Visualize.tsx'

type OptionType = 'Call' | 'Put'
type InstrumentType = 'European' | 'American'
type PriceModel = 'Standard' | 'BAW'

interface PriceResponse {
  price: string
  currency: string
}

interface GreeksResponse {
  delta: number
  gamma: number
  theta: number
  vega: number
  rho: number
}

interface SecondOrderGreeksResponse {
  vanna: number
  charm: number
  vomma: number
  speed: number
}

interface FormState {
  instrument: InstrumentType
  optionType: OptionType
  priceModel: PriceModel
  strike: string
  spot: string
  riskFreeRate: string
  volatility: string
  timeToMaturity: string
  dividendYield: string
}

const initialForm: FormState = {
  instrument: 'European',
  optionType: 'Call',
  priceModel: 'BAW',
  strike: '100',
  spot: '105',
  riskFreeRate: '5.0',
  volatility: '20.0',
  timeToMaturity: '1.0',
  dividendYield: '0.0',
}

function formatNumber(n: number): string {
  return n.toLocaleString('en-US', { maximumFractionDigits: 6 })
}

export default function OptionPricer() {
  const [form, setForm] = useState<FormState>(initialForm)
  const [activeTab, setActiveTab] = useState<'price' | 'greeks' | 'visualize'>('price')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [priceResult, setPriceResult] = useState<PriceResponse | null>(null)
  const [greeksResult, setGreeksResult] = useState<GreeksResponse | null>(null)
  const [secondOrderResult, setSecondOrderResult] = useState<SecondOrderGreeksResponse | null>(null)
  const [showSecondOrder, setShowSecondOrder] = useState(false)

  const update = (key: keyof FormState, value: string) => {
    setForm((prev) => ({ ...prev, [key]: value }))
    setError(null)
  }

  const buildBody = () => ({
    strike: form.strike,
    spot: form.spot,
    risk_free_rate: (parseFloat(form.riskFreeRate) / 100).toString(),
    volatility: (parseFloat(form.volatility) / 100).toString(),
    time_to_maturity: parseFloat(form.timeToMaturity),
    option_type: form.optionType,
    ...(form.instrument === 'American' && form.dividendYield
      ? { dividend_yield: (parseFloat(form.dividendYield) / 100).toString() }
      : {}),
  })

  const handlePrice = async () => {
    setLoading(true)
    setError(null)
    setPriceResult(null)
    setGreeksResult(null)
    setSecondOrderResult(null)
    try {
      let endpoint: string
      if (form.instrument === 'European') {
        endpoint = '/api/price/european-option'
      } else {
        endpoint =
          form.priceModel === 'BAW'
            ? '/api/price/baw-american-option'
            : '/api/price/american-option'
      }
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(buildBody()),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      const data: PriceResponse = await res.json()
      setPriceResult(data)
    } catch (e: any) {
      setError(e.message || 'Request failed')
    } finally {
      setLoading(false)
    }
  }

  const handleGreeks = async () => {
    setLoading(true)
    setError(null)
    setPriceResult(null)
    setGreeksResult(null)
    setSecondOrderResult(null)
    try {
      const endpoint =
        form.instrument === 'European'
          ? '/api/greeks/european-option'
          : '/api/greeks/american-option'
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(buildBody()),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      const data: GreeksResponse = await res.json()
      setGreeksResult(data)
    } catch (e: any) {
      setError(e.message || 'Request failed')
    } finally {
      setLoading(false)
    }
  }

  const fetchSecondOrderGreeks = async () => {
    if (secondOrderResult) return
    setLoading(true)
    setError(null)
    try {
      const endpoint =
        form.instrument === 'European'
          ? '/api/greeks/second-order/european-option'
          : '/api/greeks/second-order/american-option'
      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(buildBody()),
      })
      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }
      const data: SecondOrderGreeksResponse = await res.json()
      setSecondOrderResult(data)
    } catch (e: any) {
      setError(e.message || 'Request failed')
    } finally {
      setLoading(false)
    }
  }

  const handleSubmit = () => {
    if (activeTab === 'price') handlePrice()
    else if (activeTab === 'greeks') handleGreeks()
  }

  const inputClass =
    'w-full rounded-lg border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500'

  const labelClass = 'mb-1 block text-xs font-medium text-slate-300'

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      <header className="mb-8 text-center">
        <h1 className="text-3xl font-bold text-white">Pricing Platform</h1>
        <p className="mt-2 text-sm text-slate-400">
          Options pricing & Greeks calculator
        </p>
      </header>

      {/* Tabs */}
      <div className="mb-6 flex rounded-lg bg-slate-800 p-1">
        <button
          onClick={() => setActiveTab('price')}
          className={`flex-1 rounded-md py-2 text-sm font-medium transition-colors ${
            activeTab === 'price'
              ? 'bg-blue-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          Price
        </button>
        <button
          onClick={() => setActiveTab('greeks')}
          className={`flex-1 rounded-md py-2 text-sm font-medium transition-colors ${
            activeTab === 'greeks'
              ? 'bg-blue-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          Greeks
        </button>
        <button
          onClick={() => setActiveTab('visualize')}
          className={`flex-1 rounded-md py-2 text-sm font-medium transition-colors ${
            activeTab === 'visualize'
              ? 'bg-blue-600 text-white'
              : 'text-slate-400 hover:text-white'
          }`}
        >
          Visualize
        </button>
      </div>

      {activeTab !== 'visualize' && (
        <>
          {/* Form */}
          <div className="rounded-xl border border-slate-700 bg-slate-800/50 p-6 shadow-lg">
            <div className="mb-4 grid grid-cols-2 gap-4">
              <div>
                <label className={labelClass}>Instrument</label>
                <select
                  value={form.instrument}
                  onChange={(e) => update('instrument', e.target.value as InstrumentType)}
                  className={inputClass}
                >
                  <option value="European">European</option>
                  <option value="American">American</option>
                </select>
              </div>
              <div>
                <label className={labelClass}>Option Type</label>
                <select
                  value={form.optionType}
                  onChange={(e) => update('optionType', e.target.value as OptionType)}
                  className={inputClass}
                >
                  <option value="Call">Call</option>
                  <option value="Put">Put</option>
                </select>
              </div>
            </div>

            {activeTab === 'price' && form.instrument === 'American' && (
              <div className="mb-4">
                <label className={labelClass}>Pricing Model</label>
                <select
                  value={form.priceModel}
                  onChange={(e) => update('priceModel', e.target.value as PriceModel)}
                  className={inputClass}
                >
                  <option value="BAW">Barone-Adesi-Whaley (BAW)</option>
                  <option value="Standard">Standard American</option>
                </select>
              </div>
            )}

            <div className="mb-4 grid grid-cols-2 gap-4">
              <div>
                <label className={labelClass}>Strike Price</label>
                <input
                  type="number"
                  step="0.01"
                  value={form.strike}
                  onChange={(e) => update('strike', e.target.value)}
                  className={inputClass}
                />
              </div>
              <div>
                <label className={labelClass}>Spot Price</label>
                <input
                  type="number"
                  step="0.01"
                  value={form.spot}
                  onChange={(e) => update('spot', e.target.value)}
                  className={inputClass}
                />
              </div>
            </div>

            <div className="mb-4 grid grid-cols-2 gap-4">
              <div>
                <label className={labelClass}>Risk-Free Rate (%)</label>
                <input
                  type="number"
                  step="0.01"
                  value={form.riskFreeRate}
                  onChange={(e) => update('riskFreeRate', e.target.value)}
                  className={inputClass}
                />
              </div>
              <div>
                <label className={labelClass}>Volatility (%)</label>
                <input
                  type="number"
                  step="0.01"
                  value={form.volatility}
                  onChange={(e) => update('volatility', e.target.value)}
                  className={inputClass}
                />
              </div>
            </div>

            <div className="mb-4 grid grid-cols-2 gap-4">
              <div>
                <label className={labelClass}>Time to Maturity (years)</label>
                <input
                  type="number"
                  step="0.01"
                  value={form.timeToMaturity}
                  onChange={(e) => update('timeToMaturity', e.target.value)}
                  className={inputClass}
                />
              </div>
              {form.instrument === 'American' && (
                <div>
                  <label className={labelClass}>Dividend Yield (%)</label>
                  <input
                    type="number"
                    step="0.01"
                    value={form.dividendYield}
                    onChange={(e) => update('dividendYield', e.target.value)}
                    className={inputClass}
                  />
                </div>
              )}
            </div>

            <button
              onClick={handleSubmit}
              disabled={loading}
              className="w-full rounded-lg bg-blue-600 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {loading
                ? 'Calculating...'
                : activeTab === 'price'
                ? 'Calculate Price'
                : 'Calculate Greeks'}
            </button>
          </div>

          {/* Error */}
          {error && (
            <div className="mt-4 rounded-lg border border-red-800 bg-red-900/30 px-4 py-3 text-sm text-red-300">
              {error}
            </div>
          )}

          {/* Price Result */}
          {priceResult && (
            <div className="mt-6 rounded-xl border border-emerald-800 bg-emerald-900/20 p-6">
              <h3 className="mb-2 text-sm font-medium text-emerald-300">Price Result</h3>
              <div className="flex items-baseline gap-2">
                <span className="text-3xl font-bold text-white">
                  {formatNumber(parseFloat(priceResult.price))}
                </span>
                <span className="text-sm text-emerald-400">{priceResult.currency}</span>
              </div>
              {form.instrument === 'American' && activeTab === 'price' && (
                <p className="mt-1 text-xs text-emerald-500/70">
                  Model: {form.priceModel === 'BAW' ? 'Barone-Adesi-Whaley' : 'Standard American'}
                </p>
              )}
            </div>
          )}

          {/* Greeks Result */}
          {greeksResult && (
            <div className="mt-6 rounded-xl border border-emerald-800 bg-emerald-900/20 p-6">
              <h3 className="mb-4 text-sm font-medium text-emerald-300">
                Greeks — {form.instrument} {form.optionType}
              </h3>
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-3">
                {(
                  [
                    ['Delta', greeksResult.delta],
                    ['Gamma', greeksResult.gamma],
                    ['Theta', greeksResult.theta],
                    ['Vega', greeksResult.vega],
                    ['Rho', greeksResult.rho],
                  ] as [string, number][]
                ).map(([name, value]) => (
                  <div
                    key={name}
                    className="rounded-lg border border-emerald-800/50 bg-emerald-950/30 px-4 py-3"
                  >
                    <div className="text-xs text-emerald-400">{name}</div>
                    <div className="mt-1 text-lg font-semibold text-white">
                      {formatNumber(value)}
                    </div>
                  </div>
                ))}
              </div>

              {/* Higher-Order Greeks */}
              <div className="mt-4">
                <button
                  onClick={() => {
                    setShowSecondOrder((prev) => !prev)
                    if (!showSecondOrder && !secondOrderResult) {
                      fetchSecondOrderGreeks()
                    }
                  }}
                  className="text-xs font-medium text-emerald-400 hover:text-emerald-300"
                >
                  {showSecondOrder ? '▼' : '▶'} Higher-Order Greeks
                </button>
                {showSecondOrder && (
                  <div className="mt-3 grid grid-cols-2 gap-4 sm:grid-cols-4">
                    {secondOrderResult ? (
                      (
                        [
                          ['Vanna', secondOrderResult.vanna],
                          ['Vomma', secondOrderResult.vomma],
                          ['Charm', secondOrderResult.charm],
                          ['Speed', secondOrderResult.speed],
                        ] as [string, number][]
                      ).map(([name, value]) => (
                        <div
                          key={name}
                          className="rounded-lg border border-emerald-800/50 bg-emerald-950/30 px-4 py-3"
                        >
                          <div className="text-xs text-emerald-400">{name}</div>
                          <div className="mt-1 text-lg font-semibold text-white">
                            {formatNumber(value)}
                          </div>
                        </div>
                      ))
                    ) : (
                      <div className="col-span-full text-xs text-emerald-500/70">
                        Loading...
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}
        </>
      )}

      {activeTab === 'visualize' && (
        <Visualize
          spot={parseFloat(form.spot)}
          riskFreeRate={parseFloat(form.riskFreeRate)}
          volatility={parseFloat(form.volatility)}
          timeToMaturity={parseFloat(form.timeToMaturity)}
          dividendYield={parseFloat(form.dividendYield)}
          instrument={form.instrument}
          currentStrike={form.strike}
        />
      )}
    </div>
  )
}