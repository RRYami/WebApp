interface SelectFieldProps {
  id: string
  label: string
  value: string
  onChange: (value: string) => void
  options: string[]
  required?: boolean
}

export default function SelectField({
  id,
  label,
  value,
  onChange,
  options,
  required = false,
}: SelectFieldProps) {
  const selectClass =
    'w-full rounded-lg border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-white focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500'

  return (
    <div>
      <label className="mb-1 block text-xs font-medium text-slate-300">
        {label}
        {required && <span className="ml-1 text-red-400">*</span>}
      </label>
      <select
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={selectClass}
      >
        {options.map((opt) => (
          <option key={opt} value={opt}>
            {opt}
          </option>
        ))}
      </select>
    </div>
  )
}
