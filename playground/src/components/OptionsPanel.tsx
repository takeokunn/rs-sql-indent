import { samples, type SqlSample } from '../data/samples';

interface OptionsPanelProps {
  uppercase: boolean;
  style: string;
  autoFormat: boolean;
  onUppercaseChange: (value: boolean) => void;
  onStyleChange: (value: string) => void;
  onAutoFormatChange: (value: boolean) => void;
  onSampleSelect: (sql: string) => void;
}

export default function OptionsPanel({
  uppercase,
  style,
  autoFormat,
  onUppercaseChange,
  onStyleChange,
  onAutoFormatChange,
  onSampleSelect,
}: OptionsPanelProps) {
  return (
    <div className="mb-6 rounded-2xl glass p-5">
      <div className="flex flex-wrap items-center gap-6">
        {/* Uppercase toggle */}
        <div className="flex items-center gap-3">
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={uppercase}
              onChange={(e) => onUppercaseChange(e.target.checked)}
            />
            <span className="slider" />
          </label>
          <span className="text-sm text-text-primary">Uppercase keywords</span>
        </div>

        {/* Style segmented control */}
        <div className="flex items-center gap-3">
          <span className="text-sm text-text-secondary">Style</span>
          <div className="segmented-control">
            <button
              className={style === 'standard' ? 'active' : ''}
              onClick={() => onStyleChange('standard')}
            >
              Standard
            </button>
            <button
              className={style === 'river' ? 'active' : ''}
              onClick={() => onStyleChange('river')}
            >
              River
            </button>
          </div>
        </div>

        {/* Auto-format toggle */}
        <div className="flex items-center gap-3">
          <label className="toggle-switch">
            <input
              type="checkbox"
              checked={autoFormat}
              onChange={(e) => onAutoFormatChange(e.target.checked)}
            />
            <span className="slider" />
          </label>
          <span className="text-sm text-text-primary">Auto-format</span>
        </div>

        {/* Samples dropdown */}
        <div className="flex items-center gap-3">
          <span className="text-sm text-text-secondary">Samples</span>
          <select
            className="rounded-lg border border-border bg-bg-tertiary px-3 py-1.5 text-sm text-text-primary outline-none transition-colors focus:border-accent-purple"
            value=""
            onChange={(e) => {
              const sample = samples.find((s: SqlSample) => s.name === e.target.value);
              if (sample) onSampleSelect(sample.sql);
            }}
          >
            <option value="" disabled>
              Choose...
            </option>
            {samples.map((s: SqlSample) => (
              <option key={s.name} value={s.name}>
                {s.name}
              </option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
}
