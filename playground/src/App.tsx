import { useState, useEffect, useCallback, useRef } from 'react';
import { ArrowDown, Github } from 'lucide-react';
import { initWasm, formatSql } from './wasm/index.ts';
import SqlInput from './components/SqlInput.tsx';
import SqlOutput from './components/SqlOutput.tsx';
import OptionsPanel from './components/OptionsPanel.tsx';
import ThemeToggle from './components/ThemeToggle.tsx';

export default function App() {
  const [input, setInput] = useState('');
  const [output, setOutput] = useState('');
  const [uppercase, setUppercase] = useState(true);
  const [style, setStyle] = useState('standard');
  const [autoFormat, setAutoFormat] = useState(false);
  const [wasmLoaded, setWasmLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [theme, setTheme] = useState<'dark' | 'light'>(
    () =>
      (document.documentElement.getAttribute('data-theme') as
        | 'dark'
        | 'light') || 'dark'
  );
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(null);

  const toggleTheme = useCallback(() => {
    setTheme((prev) => {
      const next = prev === 'dark' ? 'light' : 'dark';
      document.documentElement.setAttribute('data-theme', next);
      localStorage.setItem('theme', next);
      return next;
    });
  }, []);

  useEffect(() => {
    initWasm()
      .then(() => setWasmLoaded(true))
      .catch((err) => setError(`Failed to load WASM: ${err.message}`));
  }, []);

  const doFormat = useCallback(
    (sql: string) => {
      if (!sql.trim()) {
        setOutput('');
        return;
      }
      try {
        setError(null);
        const result = formatSql(sql, uppercase, style);
        setOutput(result);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Formatting failed');
      }
    },
    [uppercase, style]
  );

  const handleInputChange = useCallback(
    (value: string) => {
      setInput(value);
      if (autoFormat && wasmLoaded) {
        if (debounceRef.current) clearTimeout(debounceRef.current);
        debounceRef.current = setTimeout(() => doFormat(value), 300);
      }
    },
    [autoFormat, wasmLoaded, doFormat]
  );

  const handleSampleSelect = useCallback(
    (sql: string) => {
      setInput(sql);
      if (wasmLoaded) {
        doFormat(sql);
      }
    },
    [wasmLoaded, doFormat]
  );

  const handleFormat = () => {
    if (!input.trim()) {
      setError('Please enter some SQL to format.');
      setOutput('');
      return;
    }
    doFormat(input);
  };

  // Re-format when options change and auto-format is on
  useEffect(() => {
    if (autoFormat && wasmLoaded && input.trim()) {
      doFormat(input);
    }
  }, [uppercase, style, autoFormat, wasmLoaded, input, doFormat]);

  return (
    <div className="mx-auto max-w-5xl px-6 py-8">
      <header className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-text-primary">
            rs-sql-indent
          </h1>
          <p className="mt-1 text-sm text-text-secondary">
            SQL formatter powered by WebAssembly — runs entirely in your browser
          </p>
        </div>
        <div className="flex items-center gap-3">
          <a
            href="https://github.com/takeokunn/rs-sql-indent"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-center rounded-lg border border-border p-2 text-text-secondary transition-all hover:bg-glass-bg hover:text-text-primary"
            title="View on GitHub"
          >
            <Github size={18} />
          </a>
          <ThemeToggle theme={theme} onToggle={toggleTheme} />
        </div>
      </header>

      <OptionsPanel
        uppercase={uppercase}
        style={style}
        autoFormat={autoFormat}
        onUppercaseChange={setUppercase}
        onStyleChange={setStyle}
        onAutoFormatChange={setAutoFormat}
        onSampleSelect={handleSampleSelect}
      />

      <SqlInput value={input} onChange={handleInputChange} theme={theme} />

      <div className="flex items-center justify-center gap-4 py-4">
        <div className="h-px flex-1 bg-border" />
        <button
          className="flex items-center gap-2 rounded-xl px-6 py-3 font-semibold text-white shadow-lg transition-all gradient-accent hover:scale-105 hover:shadow-xl disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:scale-100"
          onClick={handleFormat}
          disabled={!wasmLoaded}
        >
          {wasmLoaded ? (
            <>
              <ArrowDown size={18} />
              Format
            </>
          ) : (
            'Loading WASM...'
          )}
        </button>
        <div className="h-px flex-1 bg-border" />
      </div>

      <SqlOutput value={output} theme={theme} />

      {error && (
        <div className="mt-4 rounded-xl glass border-accent-red/30 px-4 py-3 text-sm text-error">
          {error}
        </div>
      )}

      <p className="mt-8 text-center text-xs text-text-secondary">
        Runs entirely in your browser via WebAssembly. No data is sent to any
        server.
      </p>
    </div>
  );
}
