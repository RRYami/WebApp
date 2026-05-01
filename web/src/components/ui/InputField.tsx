interface InputFieldProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  type?: 'text' | 'number'
  step?: string
  required?: boolean
  unit?: string
  displayAsPercentage?: boolean
  placeholder?: string
}

export default function InputField({
  id,
  label,
  value,
  onChange,
  type = 'text',
  step = '0.01',
  required = false,
  unit,
  displayAsPercentage,
  placeholder,
}: InputFieldProps) {
  const inputClass =
    'w-full rounded-lg border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-white placeholder-slate-400 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500'

  return (
    <div>
      <label className="mb-1 block text-xs font-medium text-slate-300">
        {label}
        {required && <span className="ml-1 text-red-400">*</span>}
        {displayAsPercentage && (
          <span className="ml-1 text-slate-500">(%)</span>
        )}
        {unit && !displayAsPercentage && (
          <span className="ml-1 text-slate-500">({unit})</span>
        )}
      </label>
      <input
        id={id}
        type={type}
        step={step}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className={inputClass}
      />
    </div>
  )
}
