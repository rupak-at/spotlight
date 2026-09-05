import { useState } from 'react';
import { ArrowLeft, Plus, Trash2, RefreshCw, Check } from 'lucide-react';
import { api } from './api';
import type { Settings, Status } from './types';

export function SettingsPanel({ settings, status, onSave, onBack }: { settings: Settings; status?: Status; onSave: (settings: Settings) => void; onBack: () => void }) {
  const [draft, setDraft] = useState(settings);
  const [newRoot, setNewRoot] = useState('');
  const [error, setError] = useState('');
  const [saving, setSaving] = useState(false);
  const [tab, setTab] = useState<'general' | 'search'>('general');
  const [exclusions, setExclusions] = useState(settings.excluded_names.join(', '));
  const update = <K extends keyof Settings>(key: K, value: Settings[K]) => setDraft(previous => ({ ...previous, [key]: value }));
  async function save() {
    setSaving(true); setError('');
    try { onSave(await api.save({ ...draft, excluded_names: exclusions.split(',').map(s => s.trim()).filter(Boolean) })); onBack(); }
    catch (error) { setError(String(error)); }
    finally { setSaving(false); }
  }
  return <section className="settings-page" aria-label="Settings">
    <header className="settings-heading"><button className="icon-button" aria-label="Back to search" onClick={onBack}><ArrowLeft size={19} /></button><div><h1>Make it yours</h1><p>A little more personal. Just as fast.</p></div></header>
    <nav className="settings-tabs" aria-label="Settings sections">{(['general', 'search'] as const).map(item => <button key={item} aria-pressed={tab === item} onClick={() => setTab(item)}>{item === 'general' ? 'Appearance & shortcuts' : 'Search & indexing'}</button>)}</nav>
    <div className="settings-content">
      {tab === 'general' ? <>
        <div className="setting-block"><label htmlFor="theme">Appearance</label><p>Choose the backdrop for your next idea.</p><div className="theme-options" id="theme">{(['dark', 'light', 'system'] as const).map(theme => <button key={theme} className={`theme-card ${theme}`} aria-pressed={draft.theme === theme} onClick={() => update('theme', theme)}><span className="theme-preview"><span /><span /><span /></span><span>{theme[0].toUpperCase() + theme.slice(1)} {draft.theme === theme && <Check size={13} />}</span></button>)}</div></div>
        <div className="setting-row"><div><label htmlFor="accent">Accent color</label><p>A subtle touch, everywhere.</p></div><input id="accent" aria-label="Accent color" type="color" value={draft.accent} onChange={e => update('accent', e.target.value)} /></div>
        <div className="setting-row"><div><label htmlFor="compact">Compact results</label><p>Fit more into your view.</p></div><input id="compact" type="checkbox" role="switch" checked={draft.compact} onChange={e => update('compact', e.target.checked)} /></div>
        <div className="setting-row"><div><label htmlFor="shortcut">Open Spotlight</label><p>For example: Super+Space or Ctrl+Alt+Space.</p></div><input id="shortcut" className="shortcut-input" value={draft.shortcut} onChange={e => update('shortcut', e.target.value)} /></div>
        <div className="settings-note">On GNOME, reassign “Switch to next input source” before using Super + Space. On Wayland, create a custom shortcut for <code>spotlight --toggle</code>.</div>
      </> : <>
        <div className="setting-block"><label htmlFor="new-root">Search folders</label><p>Only these folders and their contents are indexed. Leave empty to search apps only.</p><div className="root-list">{draft.roots.map(root => <div key={root}><span title={root}>{root}</span><button className="icon-button" aria-label={`Remove ${root}`} onClick={() => update('roots', draft.roots.filter(item => item !== root))}><Trash2 size={15} /></button></div>)}</div><div className="add-root"><input id="new-root" placeholder="/home/you/Documents" value={newRoot} onChange={e => setNewRoot(e.target.value)} /><button className="secondary-button" disabled={!newRoot.trim()} onClick={() => { const root = newRoot.trim(); if (!draft.roots.includes(root)) update('roots', [...draft.roots, root]); setNewRoot(''); }}><Plus size={16} /> Add</button></div></div>
        <div className="setting-block"><label htmlFor="excluded">Excluded names</label><p>Comma-separated file or folder names.</p><input id="excluded" value={exclusions} onChange={e => setExclusions(e.target.value)} /></div>
        <div className="setting-row"><div><label htmlFor="hidden">Include hidden items</label><p>Explicit exclusions still apply.</p></div><input id="hidden" type="checkbox" role="switch" checked={draft.include_hidden} onChange={e => update('include_hidden', e.target.checked)} /></div>
        <div className="number-settings">{([{ key: 'result_limit', label: 'Results', min: 1, max: 100 }, { key: 'max_depth', label: 'Folder depth', min: 1, max: 32 }, { key: 'max_entries', label: 'Index limit', min: 100, max: 200000 }] as const).map(item => <label key={item.key}>{item.label}<input type="number" min={item.min} max={item.max} value={draft[item.key]} onChange={e => update(item.key, Number(e.target.value))} /></label>)}</div>
        <div className="index-info"><div><strong>{status?.total.toLocaleString() ?? '—'} items indexed</strong><p>{status?.watched_directories.toLocaleString() ?? 0} folders watched · {Math.round(status?.last_scan_ms ?? 0)} ms last scan</p></div><button className="secondary-button" onClick={() => { void api.refresh().catch(error => setError(String(error))); }}><RefreshCw size={14} /> Refresh</button></div>
      </>}
      {error && <p className="error-message" role="alert">{error}</p>}
    </div>
    <footer className="settings-footer"><span>Settings stay on this device.</span><button className="primary-button" onClick={() => void save()} disabled={saving}>{saving ? 'Saving…' : 'Save changes'}</button></footer>
  </section>;
}
