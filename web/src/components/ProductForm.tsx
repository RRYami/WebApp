import { useProductContext } from '../context/ProductContext.tsx'
import Card from './ui/Card.tsx'
import InputField from './ui/InputField.tsx'
import SelectField from './ui/SelectField.tsx'

export default function ProductForm() {
  const { selectedProduct, formValues, setParam } = useProductContext()

  if (!selectedProduct) return null

  return (
    <Card title="Parameters">
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        {selectedProduct.parameters.map((param) => {
          if (param.type === 'choice' && param.options) {
            return (
              <SelectField
                key={param.id}
                id={param.id}
                label={param.label}
                value={formValues[param.id] || ''}
                onChange={(v) => setParam(param.id, v)}
                options={param.options}
                required={param.required}
              />
            )
          }

          return (
            <InputField
              key={param.id}
              id={param.id}
              label={param.label}
              value={formValues[param.id] || ''}
              onChange={(v) => setParam(param.id, v)}
              type="number"
              step="0.01"
              required={param.required}
              unit={param.unit}
              displayAsPercentage={param.display_as_percentage}
              placeholder={param.display_as_percentage ? '5.0' : undefined}
            />
          )
        })}
      </div>
    </Card>
  )
}
