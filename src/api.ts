import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Entry, Filter, SearchResponse, Settings, Status } from './types';

export const desktop = isTauri();
const previewSettings: Settings = {
  roots: ['/home/you/Documents', '/home/you/projects'],
  excluded_names: ['node_modules', 'target', 'dist', '.git', '.cache', '.venv'],
  include_hidden: false, max_entries: 50000, max_depth: 12, result_limit: 30,
  shortcut: 'Super+Space', theme: 'dark', accent: '#a5b4fc', compact: false,
};
// Browser preview is deliberately separate from the native API and labeled in the UI.
const samples: Entry[] = [
  { id: 'p1', name: 'Files', path: 'org.gnome.Nautilus.desktop', kind: 'app', keywords: 'Browse and organize your files' },
  { id: 'p2', name: 'Firefox', path: 'firefox.desktop', kind: 'app', keywords: 'Web browser' },
  { id: 'p3', name: 'Terminal', path: 'org.gnome.Terminal.desktop', kind: 'app', keywords: 'Command line' },
  { id: 'p4', name: 'Visual Studio Code', path: 'code.desktop', kind: 'app', keywords: 'Code editor' },
  { id: 'p5', name: 'Documents', path: '/home/you/Documents', kind: 'folder', keywords: '' },
  { id: 'p6', name: 'spotlight', path: '/home/you/projects/spotlight', kind: 'folder', keywords: '' },
  { id: 'p7', name: 'architecture.md', path: '/home/you/projects/spotlight/docs/architecture.md', kind: 'file', keywords: '' },
];
let currentPreviewSettings = { ...previewSettings };
export const api = {
  settings: () => desktop ? invoke<Settings>('get_settings') : Promise.resolve(currentPreviewSettings),
  status: () => desktop ? invoke<Status>('get_status') : Promise.resolve<Status>({ indexing: false, total: samples.length, apps: 4, truncated: false, warnings: [], shortcut_message: null, shortcut_active: false, session: 'browser preview', last_scan_ms: 0, watched_directories: 0 }),
  search: (query: string, filter: Filter) => desktop
    ? invoke<SearchResponse>('search', { query, kind: filter === 'all' ? null : filter })
    : Promise.resolve({ results: samples.filter(e => (filter === 'all' || e.kind === filter) && `${e.name} ${e.path} ${e.keywords}`.toLowerCase().includes(query.toLowerCase().trim())), elapsed_ms: 0, total: samples.length }),
  save: (settings: Settings) => {
    if (desktop) return invoke<Settings>('save_settings', { settings });
    currentPreviewSettings = settings;
    return Promise.resolve(settings);
  },
  launch: (id: string) => desktop ? invoke<void>('launch', { id }) : Promise.reject(new Error('This is a browser preview. Run npm run desktop to open apps and files.')),
  hide: () => desktop ? invoke<void>('hide_window') : Promise.resolve(),
  quit: () => desktop ? invoke<void>('quit') : Promise.resolve(),
  refresh: () => desktop ? invoke<void>('refresh_index') : Promise.resolve(),
  on: async (event: string, callback: () => void) => desktop ? listen(event, callback) : () => {},
};
