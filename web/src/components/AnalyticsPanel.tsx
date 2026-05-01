import { useProductContext } from '../context/ProductContext.tsx'
import Card from './ui/Card.tsx'
import Button from './ui/Button.tsx'
import CurveConfigPanel from './CurveConfigPanel.tsx'

const ANALYTICS_LABELS: Record<string, string> = {
  price: 'Calculate Price',
  greeks: 'Calculate Greeks',
  'second-order-greeks': 'Calculate Higher-Order Greeks',
  curve: 'Generate Curve',
}

export default function AnalyticsPanel() {
  const { selectedProduct, loading, runAnalytics, showCurveConfig, setShowCurveConfig } = useProductContext()

  if (!selectedProduct) return null

  const handleClick = (id: string) => {
    if (id === 'curve') {
      setShowCurveConfig(true)
    } else {
      runAnalytics(id)
    }
  }

  return (
    <>
      <Card title="Analytics">
        <div className="flex flex-wrap gap-3">
          {selectedProduct.analytics.map((id) => (
            <Button
              key={id}
              variant="primary"
              loading={loading}
              onClick={() => handleClick(id)}
            >
              {ANALYTICS_LABELS[id] || id}
            </Button>
          ))}
        </div>
      </Card>
      {showCurveConfig && <CurveConfigPanel />}
    </>
  )
}
