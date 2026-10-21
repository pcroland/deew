import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

export interface SelectChoice {
  value: string
  label: string
}

export function SimpleSelect({
  id,
  value,
  onChange,
  choices,
  disabled,
}: {
  id?: string
  value: string
  onChange: (value: string) => void
  choices: SelectChoice[]
  disabled?: boolean
}) {
  return (
    <Select
      value={value}
      onValueChange={(v) => {
        if (v !== null) onChange(String(v))
      }}
      items={choices}
      disabled={disabled}
    >
      <SelectTrigger id={id} size="sm" className="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent alignItemWithTrigger={false}>
        {choices.map((c) => (
          <SelectItem key={c.value} value={c.value}>
            {c.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
