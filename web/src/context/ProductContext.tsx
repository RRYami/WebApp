import React, { createContext, useContext, useState, useCallback } from 'react'

export interface ProductParameter {
  id: string
  label: string
  type: string
  required: boolean
  display_as_percentage?: boolean
  unit?: string
  options?: string[]
}

export interface ProductSchema {
  id: string
  name: string
  category: string
  parameters: ProductParameter[]
  analytics: string[]
}

export interface CurveConfig {
  min: string
  max: string
  steps: string
  curveType: 'price-vs-strike' | 'greeks-vs-spot'
}

interface ProductContextType {
  products: ProductSchema[] | null
  selectedProduct: ProductSchema | null
  formValues: Record<string, string>
  activeAnalytics: string | null
  results: Record<string, unknown> | null
  loading: boolean
  error: string | null
  showCurveConfig: boolean
  curveConfig: CurveConfig
  loadProducts: () => Promise<void>
  selectProduct: (id: string) => void
  clearProduct: () => void
  setParam: (id: string, value: string) => void
  runAnalytics: (analyticsId: string) => Promise<void>
  setShowCurveConfig: (show: boolean) => void
  setCurveConfig: (config: Partial<CurveConfig>) => void
  generateCurve: () => Promise<void>
  clearResults: () => void
}

const ProductContext = createContext<ProductContextType | undefined>(undefined)

export function ProductProvider({ children }: { children: React.ReactNode }) {
  const [products, setProducts] = useState<ProductSchema[] | null>(null)
  const [selectedProduct, setSelectedProduct] = useState<ProductSchema | null>(null)
  const [formValues, setFormValues] = useState<Record<string, string>>({})
  const [activeAnalytics, setActiveAnalytics] = useState<string | null>(null)
  const [results, setResults] = useState<Record<string, unknown> | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [showCurveConfig, setShowCurveConfig] = useState(false)
  const [curveConfig, setCurveConfigState] = useState<CurveConfig>({
    min: '',
    max: '',
    steps: '30',
    curveType: 'price-vs-strike',
  })

  const loadProducts = useCallback(async () => {
    try {
      const res = await fetch('/api/products')
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const data = await res.json()
      setProducts(data.products)
    } catch (e: any) {
      setError(e.message || 'Failed to load products')
    }
  }, [])

  const selectProduct = useCallback(
    (id: string) => {
      const product = products?.find((p) => p.id === id) || null
      setSelectedProduct(product)
      // Initialise form with empty values for all parameters
      const initial: Record<string, string> = {}
      product?.parameters.forEach((param) => {
        if (param.type === 'choice' && param.options && param.options.length > 0) {
          initial[param.id] = param.options[0]
        } else {
          initial[param.id] = ''
        }
      })
      setFormValues(initial)
      setResults(null)
      setActiveAnalytics(null)
      setError(null)
      setShowCurveConfig(false)
      // Initialise curve config defaults from spot
      const spot = initial['spot'] || '100'
      const spotNum = parseFloat(spot) || 100
      setCurveConfigState({
        min: (spotNum * 0.7).toFixed(2),
        max: (spotNum * 1.3).toFixed(2),
        steps: '30',
        curveType: 'price-vs-strike',
      })
    },
    [products]
  )

  const clearProduct = useCallback(() => {
    setSelectedProduct(null)
    setFormValues({})
    setResults(null)
    setActiveAnalytics(null)
    setError(null)
  }, [])

  const setParam = useCallback((id: string, value: string) => {
    setFormValues((prev) => ({ ...prev, [id]: value }))
    setError(null)
  }, [])

  const runAnalytics = useCallback(
    async (analyticsId: string) => {
      if (!selectedProduct) return
      setLoading(true)
      setError(null)
      setResults(null)
      setActiveAnalytics(analyticsId)

      try {
        // Build parameters: convert percentage fields
        const parameters: Record<string, string | number> = {}
        selectedProduct.parameters.forEach((param) => {
          const raw = formValues[param.id] || ''
          if (raw === '' && !param.required) {
            return // skip optional empty fields
          }
          if (param.display_as_percentage && raw !== '') {
            parameters[param.id] = (parseFloat(raw) / 100).toString()
          } else if (param.type === 'float' && raw !== '') {
            parameters[param.id] = parseFloat(raw)
          } else {
            parameters[param.id] = raw
          }
        })

        let endpoint: string
        let body: Record<string, unknown>

        switch (analyticsId) {
          case 'price':
            endpoint = '/api/analytics/price'
            body = { product: selectedProduct.id, parameters }
            break
          case 'greeks':
            endpoint = '/api/analytics/greeks'
            body = { product: selectedProduct.id, parameters }
            break
          case 'second-order-greeks':
            endpoint = '/api/analytics/second-order-greeks'
            body = { product: selectedProduct.id, parameters }
            break
          case 'curve': {
            endpoint = '/api/analytics/curve/price'
            const spot = parseFloat(formValues['spot'] || '100')
            const min = Math.round(spot * 0.7)
            const max = Math.round(spot * 1.3)
            const steps = 30
            const step = (max - min) / (steps - 1)
            const strikes = Array.from({ length: steps }, (_, i) =>
              (min + step * i).toFixed(4)
            )
            body = {
              product: selectedProduct.id,
              parameters,
              strikes,
            }
            break
          }
          default:
            throw new Error(`Unknown analytics: ${analyticsId}`)
        }

        const res = await fetch(endpoint, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(body),
        })

        if (!res.ok) {
          const err = await res.json()
          throw new Error(err.error || `HTTP ${res.status}`)
        }

        const data = await res.json()
        setResults(data)
      } catch (e: any) {
        setError(e.message || 'Request failed')
      } finally {
        setLoading(false)
      }
    },
    [selectedProduct, formValues]
  )

  const setCurveConfig = useCallback((config: Partial<CurveConfig>) => {
    setCurveConfigState((prev) => ({ ...prev, ...config }))
  }, [])

  const generateCurve = useCallback(async () => {
    if (!selectedProduct) return
    setLoading(true)
    setError(null)
    setResults(null)
    setActiveAnalytics('curve')
    setShowCurveConfig(false)

    try {
      // Build parameters: convert percentage fields
      const parameters: Record<string, string | number> = {}
      selectedProduct.parameters.forEach((param) => {
        const raw = formValues[param.id] || ''
        if (raw === '' && !param.required) {
          return
        }
        if (param.display_as_percentage && raw !== '') {
          parameters[param.id] = (parseFloat(raw) / 100).toString()
        } else if (param.type === 'float' && raw !== '') {
          parameters[param.id] = parseFloat(raw)
        } else {
          parameters[param.id] = raw
        }
      })

      const min = parseFloat(curveConfig.min)
      const max = parseFloat(curveConfig.max)
      const steps = parseInt(curveConfig.steps, 10)

      if (isNaN(min) || isNaN(max) || isNaN(steps) || steps < 2) {
        throw new Error('Invalid curve configuration: min, max, and steps must be valid numbers (steps >= 2)')
      }

      const step = (max - min) / (steps - 1)
      const values = Array.from({ length: steps }, (_, i) =>
        (min + step * i).toFixed(4)
      )

      let endpoint: string
      let body: Record<string, unknown>

      if (curveConfig.curveType === 'price-vs-strike') {
        endpoint = '/api/analytics/curve/price'
        body = {
          product: selectedProduct.id,
          parameters,
          strikes: values,
        }
      } else {
        endpoint = '/api/analytics/curve/greeks'
        body = {
          product: selectedProduct.id,
          parameters,
          spots: values,
          fixed_strike: parameters['strike'],
        }
      }

      const res = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      })

      if (!res.ok) {
        const err = await res.json()
        throw new Error(err.error || `HTTP ${res.status}`)
      }

      const data = await res.json()
      setResults(data)
    } catch (e: any) {
      setError(e.message || 'Request failed')
    } finally {
      setLoading(false)
    }
  }, [selectedProduct, formValues, curveConfig])

  const clearResults = useCallback(() => {
    setResults(null)
    setActiveAnalytics(null)
    setError(null)
    setShowCurveConfig(false)
  }, [])

  return (
    <ProductContext.Provider
      value={{
        products,
        selectedProduct,
        formValues,
        activeAnalytics,
        results,
        loading,
        error,
        showCurveConfig,
        curveConfig,
        loadProducts,
        selectProduct,
        clearProduct,
        setParam,
        runAnalytics,
        setShowCurveConfig,
        setCurveConfig,
        generateCurve,
        clearResults,
      }}
    >
      {children}
    </ProductContext.Provider>
  )
}

export function useProductContext() {
  const ctx = useContext(ProductContext)
  if (!ctx) {
    throw new Error('useProductContext must be used within ProductProvider')
  }
  return ctx
}
