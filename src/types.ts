export type Kind = 'app' | 'file' | 'folder';
export type Filter = Kind | 'all';
export interface Entry { id: string; name: string; path: string; kind: Kind; keywords: string }
export interface SearchResponse { results: Entry[]; elapsed_ms: number; total: number }
export interface Settings {
  roots: string[]; excluded_names: string[]; include_hidden: boolean;
  max_entries: number; max_depth: number; result_limit: number;
  shortcut: string; theme: 'dark' | 'light' | 'system'; accent: string; compact: boolean;
}
export interface Status {
  indexing: boolean; total: number; apps: number; truncated: boolean;
  warnings: string[]; shortcut_message: string | null; shortcut_active: boolean;
  session: string; last_scan_ms: number; watched_directories: number;
}
