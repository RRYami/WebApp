import { useProductContext } from '../context/ProductContext.tsx'
import ProductForm from './ProductForm.tsx'
import AnalyticsPanel from './AnalyticsPanel.tsx'
import ResultsPanel from './ResultsPanel.tsx'

export default function ProductWorkspace() {
  const { selectedProduct, clearProduct } = useProductContext()

  if (!selectedProduct) return null

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      <header className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">{selectedProduct.name}</h1>
          <p className="mt-1 text-sm text-slate-400">{selectedProduct.category}</p>
        </div>
        <button
          onClick={clearProduct}
          className="rounded-lg border border-slate-600 bg-slate-800 px-4 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
        >
          Change Product
        </button>
      </header>

      <div className="space-y-6">
        <ProductForm />
        <AnalyticsPanel />
        <ResultsPanel />
      </div>
    </div>
  )
}
