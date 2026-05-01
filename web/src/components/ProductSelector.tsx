import { useEffect, useMemo } from 'react'
import { useProductContext, type ProductSchema } from '../context/ProductContext.tsx'

export default function ProductSelector() {
  const { products, loadProducts, selectProduct, error } = useProductContext()

  useEffect(() => {
    if (!products) {
      loadProducts()
    }
  }, [products, loadProducts])

  // Group products by category
  const grouped = useMemo<Record<string, ProductSchema[]>>(() => {
    if (!products) return {}
    const map: Record<string, ProductSchema[]> = {}
    products.forEach((p) => {
      if (!map[p.category]) map[p.category] = []
      map[p.category].push(p)
    })
    return map
  }, [products])

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      <header className="mb-8 text-center">
        <h1 className="text-3xl font-bold text-white">Pricing Platform</h1>
        <p className="mt-2 text-sm text-slate-400">
          Select a product to begin pricing and risk analysis
        </p>
      </header>

      {error && (
        <div className="mb-4 rounded-lg border border-red-800 bg-red-900/30 px-4 py-3 text-sm text-red-300">
          {error}
        </div>
      )}

      <div className="space-y-8">
        {Object.entries(grouped).map(([category, items]) => (
          <div key={category}>
            <h2 className="mb-4 text-lg font-semibold text-white">{category}</h2>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3">
              {items.map((product: ProductSchema) => (
                <button
                  key={product.id}
                  onClick={() => selectProduct(product.id)}
                  className="group rounded-xl border border-slate-700 bg-slate-800/50 p-6 text-left shadow-lg transition-colors hover:border-blue-500 hover:bg-slate-800"
                >
                  <h3 className="text-base font-semibold text-white group-hover:text-blue-400">
                    {product.name}
                  </h3>
                  <p className="mt-2 text-xs text-slate-400">
                    {product.analytics.length} analytics available
                  </p>
                  <div className="mt-3 flex flex-wrap gap-1.5">
                    {product.analytics.slice(0, 4).map((a: string) => (
                      <span
                        key={a}
                        className="rounded-full bg-slate-700 px-2 py-0.5 text-[10px] text-slate-300"
                      >
                        {a}
                      </span>
                    ))}
                  </div>
                </button>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
