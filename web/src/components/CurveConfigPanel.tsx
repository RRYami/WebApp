import { useProductContext } from '../context/ProductContext.tsx'
import Card from './ui/Card.tsx'
import InputField from './ui/InputField.tsx'
import Button from './ui/Button.tsx'

export default function CurveConfigPanel() {
  const {
    curveConfig,
    setCurveConfig,
    generateCurve,
    loading,
    setShowCurveConfig,
  } = useProductContext()

  return (
    <Card title="Curve Configuration">
      <div className="space-y-4">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <InputField
            id="curve-min"
            label="Min"
            value={curveConfig.min}
            onChange={(v) => setCurveConfig({ min: v })}
            type="number"
            step="0.01"
            required
          />
          <InputField
            id="curve-max"
            label="Max"
            value={curveConfig.max}
            onChange={(v) => setCurveConfig({ max: v })}
            type="number"
            step="0.01"
            required
          />
          <InputField
            id="curve-steps"
            label="Steps"
            value={curveConfig.steps}
            onChange={(v) => setCurveConfig({ steps: v })}
            type="number"
            step="1"
            required
          />
        </div>

        <div>
          <label className="mb-1 block text-xs font-medium text-slate-300">
            Curve Type
          </label>
          <div className="flex gap-3">
            {(['price-vs-strike', 'greeks-vs-spot'] as const).map((type) => (
              <label
                key={type}
                className={`flex cursor-pointer items-center gap-2 rounded-lg border px-4 py-2 text-sm transition-colors ${
                  curveConfig.curveType === type
                    ? 'border-blue-500 bg-blue-500/20 text-blue-400'
                    : 'border-slate-600 bg-slate-800 text-slate-300'
                }`}
              >
                <input
                  type="radio"
                  name="curveType"
                  value={type}
                  checked={curveConfig.curveType === type}
                  onChange={() => setCurveConfig({ curveType: type })}
                  className="hidden"
                />
                <span>
                  {type === 'price-vs-strike'
                    ? 'Price vs Strike'
                    : 'Greeks vs Spot'}
                </span>
              </label>
            ))}
          </div>
        </div>

        <div className="flex gap-3">
          <Button
            variant="primary"
            loading={loading}
            onClick={() => generateCurve()}
          >
            Generate
          </Button>
          <Button
            variant="secondary"
            onClick={() => setShowCurveConfig(false)}
          >
            Cancel
          </Button>
        </div>
      </div>
    </Card>
  )
}
