import CodeMirror from '@uiw/react-codemirror';
import { sql } from '@codemirror/lang-sql';
import { dracula } from '@uiw/codemirror-theme-dracula';
import { githubLight } from '@uiw/codemirror-theme-github';
import { useCallback, DragEvent } from 'react';

interface SqlInputProps {
  value: string;
  onChange: (value: string) => void;
  theme: 'dark' | 'light';
}

export default function SqlInput({ value, onChange, theme }: SqlInputProps) {
  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const file = e.dataTransfer.files[0];
      if (file && file.name.endsWith('.sql')) {
        const reader = new FileReader();
        reader.onload = (ev) => {
          const text = ev.target?.result;
          if (typeof text === 'string') {
            onChange(text);
          }
        };
        reader.readAsText(file);
      }
    },
    [onChange]
  );

  return (
    <div className="flex flex-col gap-2">
      <label className="text-xs font-semibold uppercase tracking-wide text-text-secondary">
        Input SQL
      </label>
      <div
        className="overflow-hidden rounded-xl border border-border glass"
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        <CodeMirror
          value={value}
          height="400px"
          theme={theme === 'dark' ? dracula : githubLight}
          extensions={[sql()]}
          onChange={onChange}
          placeholder="Paste your SQL here or drag a .sql file..."
          basicSetup={{
            lineNumbers: true,
            foldGutter: false,
            highlightActiveLine: true,
          }}
        />
      </div>
    </div>
  );
}
