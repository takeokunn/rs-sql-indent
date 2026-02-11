import { useState, useCallback } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { sql } from '@codemirror/lang-sql';
import { dracula } from '@uiw/codemirror-theme-dracula';
import { githubLight } from '@uiw/codemirror-theme-github';
import { Copy, Check, Download } from 'lucide-react';

interface SqlOutputProps {
  value: string;
  theme: 'dark' | 'light';
}

export default function SqlOutput({ value, theme }: SqlOutputProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    if (!value) return;
    await navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [value]);

  const handleDownload = useCallback(() => {
    if (!value) return;
    const blob = new Blob([value], { type: 'text/sql' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'formatted.sql';
    a.click();
    URL.revokeObjectURL(url);
  }, [value]);

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between">
        <label className="text-xs font-semibold uppercase tracking-wide text-text-secondary">
          Output SQL
        </label>
        <div className="flex gap-2">
          <button
            className="flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs text-text-secondary transition-all hover:bg-glass-bg hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-40"
            onClick={handleDownload}
            disabled={!value}
            title="Download SQL"
          >
            <Download size={14} />
            <span>Download</span>
          </button>
          <button
            className="flex items-center gap-1.5 rounded-lg border border-border px-3 py-1.5 text-xs text-text-secondary transition-all hover:bg-glass-bg hover:text-text-primary disabled:cursor-not-allowed disabled:opacity-40"
            onClick={handleCopy}
            disabled={!value}
            title="Copy to clipboard"
          >
            {copied ? <Check size={14} className="text-accent-green" /> : <Copy size={14} />}
            <span>{copied ? 'Copied!' : 'Copy'}</span>
          </button>
        </div>
      </div>
      <div className="overflow-hidden rounded-xl border border-border glass">
        <CodeMirror
          value={value}
          height="400px"
          theme={theme === 'dark' ? dracula : githubLight}
          extensions={[sql()]}
          editable={false}
          readOnly={true}
          basicSetup={{
            lineNumbers: true,
            foldGutter: false,
            highlightActiveLine: false,
          }}
        />
      </div>
    </div>
  );
}
