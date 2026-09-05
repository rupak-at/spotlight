import { useEffect, useRef, useState } from 'react';
import {
  ArrowDown,
  ArrowUp,
  Command,
  CornerDownLeft,
  Search,
  Settings2,
  X,
  RefreshCw,
  Power,
  CircleAlert,
} from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { api, desktop } from './api';
import { useSearch } from './useSearch';
import { SettingsPanel } from './SettingsPanel';
import { ResultIcon } from './ResultIcon';
import type { Entry, Filter, Settings, Status } from './types';

const filters: { key: Filter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'app', label: 'Applications' },
  { key: 'file', label: 'Files' },
  { key: 'folder', label: 'Folders' },
];
const kindLabels = { app: 'Application', file: 'File', folder: 'Folder' };

export function App() {
  const [query, setQuery] = useState('');
  const [filter, setFilter] = useState<Filter>('all');
  const [selected, setSelected] = useState(0);
  const [revision, setRevision] = useState(0);
  const [settings, setSettings] = useState<Settings>();
  const [status, setStatus] = useState<Status>();
  const [showSettings, setShowSettings] = useState(false);
  const [error, setError] = useState('');
  const [launching, setLaunching] = useState(false);
  const launchingRef = useRef(false);
  const input = useRef<HTMLInputElement>(null);
  const { data, pending, error: searchError } = useSearch(query, filter, revision);
  const results = data?.results ?? [];
  const active = results[selected];

  useEffect(() => {
    let alive = true;
    const cleanups: (() => void)[] = [];
    const statusChanged = () => {
      void api
        .status()
        .then((value) => {
          if (alive) {
            setStatus(value);
            setRevision((r) => r + 1);
          }
        })
        .catch((error) => {
          if (alive) setError(String(error));
        });
    };
    void api
      .settings()
      .then((value) => {
        if (alive) setSettings(value);
      })
      .catch((error) => {
        if (alive) setError(String(error));
      });
    statusChanged();
    for (const [event, callback] of [
      ['index-changed', statusChanged],
      [
        'launcher-shown',
        () => {
          setShowSettings(false);
          setQuery('');
          setFilter('all');
          setError('');
          requestAnimationFrame(() => input.current?.focus());
        },
      ],
    ] as const) {
      void api
        .on(event, callback)
        .then((unlisten) => {
          if (alive) cleanups.push(unlisten);
          else unlisten();
        })
        .catch((error) => {
          if (alive) setError(String(error));
        });
    }
    return () => {
      alive = false;
      cleanups.forEach((cleanup) => cleanup());
    };
  }, []);

  useEffect(() => {
    setSelected(0);
  }, [query, filter, revision]);
  useEffect(() => {
    document.getElementById(`result-${selected}`)?.scrollIntoView?.({ block: 'nearest' });
  }, [selected]);
  useEffect(() => {
    document.documentElement.dataset.theme = settings?.theme ?? 'dark';
    document.documentElement.style.setProperty('--accent', settings?.accent ?? '#c1c5cf');
    document.documentElement.style.setProperty(
      '--background-opacity',
      `${settings?.background_opacity ?? 94}%`,
    );
  }, [settings]);
  useEffect(() => {
    if (!showSettings) input.current?.focus();
  }, [showSettings]);

  async function launch(entry: Entry) {
    if (launchingRef.current || pending) return;
    launchingRef.current = true;
    setLaunching(true);
    setError('');
    try {
      await api.launch(entry.id);
    } catch (error) {
      setError(String(error));
    } finally {
      launchingRef.current = false;
      setLaunching(false);
    }
  }
  const safely = (task: Promise<void>) => {
    void task.catch((error) => setError(String(error)));
  };
  function keyDown(event: React.KeyboardEvent) {
    if (event.nativeEvent.isComposing) return;
    if (event.key === 'Escape') {
      event.preventDefault();
      safely(api.hide());
      return;
    }
    if ((event.ctrlKey || event.metaKey) && event.key === ',') {
      event.preventDefault();
      setShowSettings((value) => !value);
      return;
    }
    if (showSettings) return;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      if (results.length)
        setSelected(
          (i) => (i + (event.key === 'ArrowDown' ? 1 : -1) + results.length) % results.length,
        );
    } else if (event.key === 'Enter' && event.target === input.current && active) {
      event.preventDefault();
      void launch(active);
    } else if (event.key === 'Tab' && event.target === input.current && event.ctrlKey) {
      event.preventDefault();
      const i = filters.findIndex((f) => f.key === filter);
      setFilter(filters[(i + (event.shiftKey ? 3 : 1)) % 4].key);
    }
  }
  const warnings = [
    ...(status?.shortcut_message ? [status.shortcut_message] : []),
    ...(status?.warnings ?? []),
  ];

  return (
    <main
      className={`launcher ${settings?.compact !== false ? 'compact' : ''}`}
      onKeyDown={keyDown}
    >
      <div
        className="titlebar"
        onMouseDown={(event) => {
          if (desktop && event.button === 0 && !(event.target as HTMLElement).closest('button'))
            safely(getCurrentWindow().startDragging());
        }}
      >
        <span className="wordmark">Spotlight</span>
        <div className="window-actions">
          <button
            className="icon-button"
            aria-label="Hide Spotlight"
            title="Hide Spotlight (Esc)"
            onClick={() => safely(api.hide())}
          >
            <X size={16} />
          </button>
        </div>
      </div>
      {showSettings && settings ? (
        <SettingsPanel
          settings={settings}
          status={status}
          onSave={(value) => {
            setSettings(value);
            setRevision((r) => r + 1);
          }}
          onBack={() => setShowSettings(false)}
        />
      ) : (
        <>
          <div className="search-box">
            <Search className="search-glyph" size={25} strokeWidth={1.6} />
            <input
              ref={input}
              role="combobox"
              aria-label="Search applications, files, and folders"
              aria-controls="results"
              aria-expanded={true}
              aria-activedescendant={active ? `result-${selected}` : undefined}
              aria-autocomplete="list"
              placeholder="Search anything"
              value={query}
              maxLength={256}
              spellCheck={false}
              autoComplete="off"
              autoFocus
              onChange={(event) => {
                setQuery(event.target.value);
                setError('');
              }}
            />
          </div>
          <div className="filter-row">
            <nav aria-label="Result types">
              {filters.map((item) => (
                <button
                  key={item.key}
                  aria-pressed={filter === item.key}
                  onClick={() => {
                    setFilter(item.key);
                    input.current?.focus();
                  }}
                >
                  {item.label}
                  {item.key === filter && <span className="filter-dot" />}
                </button>
              ))}
            </nav>
          </div>
          {warnings.length > 0 && (
            <details className="notice">
              <summary>
                <CircleAlert size={13} />
                {status?.shortcut_message ? 'Shortcut needs setup' : 'Index notice'}
                <span>View details</span>
              </summary>
              <div>
                {warnings.map((warning, i) => (
                  <p key={i}>{warning}</p>
                ))}
              </div>
            </details>
          )}
          {(error || searchError) && (
            <div className="error-message" role="alert">
              {error || searchError}
            </div>
          )}
          <div className="results-heading">
            <span>{query ? 'Results' : 'Suggestions'}</span>
            <span>
              {pending
                ? 'Searching…'
                : `${results.length} ${results.length === 1 ? 'result' : 'results'}`}
            </span>
          </div>
          <div
            className="results"
            id="results"
            role="listbox"
            aria-label="Search results"
            aria-busy={pending}
          >
            {results.map((entry, i) => {
              return (
                <div
                  id={`result-${i}`}
                  key={entry.id}
                  role="option"
                  aria-selected={i === selected}
                  className={`result ${i === selected ? 'selected' : ''}`}
                  onMouseMove={() => setSelected(i)}
                  onClick={() => void launch(entry)}
                >
                  <ResultIcon entry={entry} revision={revision} />
                  <div className="result-text">
                    <span className="result-name">{entry.name}</span>
                    <span className="result-path" title={entry.path}>
                      {entry.kind === 'app' ? entry.keywords || entry.path : entry.path}
                    </span>
                  </div>
                  <span className="result-kind">{kindLabels[entry.kind]}</span>
                  {i === selected && <CornerDownLeft className="result-enter" size={16} />}
                </div>
              );
            })}
            {!results.length && (
              <div className="empty-state">
                <span className="empty-icon">
                  <Search size={26} strokeWidth={1.3} />
                </span>
                <h2>
                  {pending
                    ? 'Searching…'
                    : status?.indexing
                      ? 'Indexing your files'
                      : query
                        ? 'No results'
                        : 'Ready to search'}
                </h2>
                <p>
                  {pending
                    ? ' '
                    : status?.indexing
                      ? 'Your index is being built in the background.'
                      : query
                        ? 'Try a different name, or add a search folder in settings.'
                        : 'Add folders in settings, or refresh your application index.'}
                </p>
              </div>
            )}
          </div>
          <footer className="launcher-footer">
            <div className="key-hints">
              <span>
                <kbd>
                  <ArrowUp size={11} />
                </kbd>
                <kbd>
                  <ArrowDown size={11} />
                </kbd>{' '}
                navigate
              </span>
              <span>
                <kbd>
                  <CornerDownLeft size={12} />
                </kbd>{' '}
                {launching ? 'opening…' : 'open'}
              </span>
            </div>
            <div className="footer-actions">
              <span className="index-status" aria-live="polite">
                <span className={status?.indexing ? 'status-dot indexing' : 'status-dot'} />
                {status?.indexing ? 'Indexing…' : ''}
              </span>
              <button
                className="icon-button"
                title="Refresh index"
                aria-label="Refresh index"
                onClick={() => safely(api.refresh())}
              >
                <RefreshCw size={15} className={status?.indexing ? 'spin' : ''} />
              </button>
              <button
                className="icon-button"
                title="Settings (Ctrl+,)"
                aria-label="Settings"
                disabled={!settings}
                onClick={() => setShowSettings(true)}
              >
                <Settings2 size={16} />
              </button>
              <button
                className="icon-button"
                title="Quit Spotlight"
                aria-label="Quit Spotlight"
                onClick={() => safely(api.quit())}
              >
                <Power size={15} />
              </button>
            </div>
          </footer>
        </>
      )}
      {!desktop && (
        <div className="preview-note">
          <Command size={13} /> UI preview with sample data · <code>npm run desktop</code> for local
          search
        </div>
      )}
    </main>
  );
}
