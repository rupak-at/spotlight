import { act, renderHook, waitFor } from '@testing-library/react';
import { api } from './api';
import { useSearch } from './useSearch';
import type { SearchResponse } from './types';

vi.mock('./api', () => ({ api: { search: vi.fn() } }));
const response = (name: string): SearchResponse => ({ results: [{ id: name, name, path: name, kind: 'app', keywords: '' }], elapsed_ms: 1, total: 1 });

it('clears stale results immediately and rejects a late response from an older query', async () => {
  const resolve: Record<string, (value: SearchResponse) => void> = {};
  vi.mocked(api.search).mockImplementation(query => new Promise(done => { resolve[query] = done; }));
  const { result, rerender } = renderHook(({ query }) => useSearch(query, 'all', 0), { initialProps: { query: 'fire' } });
  await waitFor(() => expect(resolve.fire).toBeDefined());
  rerender({ query: 'files' });
  expect(result.current.data).toBeUndefined();
  expect(result.current.pending).toBe(true);
  await waitFor(() => expect(resolve.files).toBeDefined());
  await act(async () => { resolve.files(response('Files')); });
  expect(result.current.data?.results[0].name).toBe('Files');
  await act(async () => { resolve.fire(response('Firefox')); });
  expect(result.current.data?.results[0].name).toBe('Files');
});

it('exposes query failures without leaving an endless loading state', async () => {
  vi.mocked(api.search).mockRejectedValueOnce(new Error('Search unavailable'));
  const { result } = renderHook(() => useSearch('test', 'file', 0));
  await waitFor(() => expect(result.current.error).toContain('Search unavailable'));
  expect(result.current.pending).toBe(false);
});
