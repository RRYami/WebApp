import { ProductProvider, useProductContext } from './context/ProductContext.tsx'
import ProductSelector from './components/ProductSelector.tsx'
import ProductWorkspace from './components/ProductWorkspace.tsx'

function AppContent() {
  const { selectedProduct } = useProductContext()
  return selectedProduct ? <ProductWorkspace /> : <ProductSelector />
}

function App() {
  return (
    <ProductProvider>
      <AppContent />
    </ProductProvider>
  )
}

export default App