import { act, renderHook, waitFor } from '@testing-library/react';
import { api } from './api';
import { useSearch } from './useSearch';
import type { SearchResponse } from './types';

vi.mock('./api', () => ({ api: { search: vi.fn() } }));
const response = (name: string): SearchResponse => ({
  results: [{ id: name, name, path: name, kind: 'app', keywords: '' }],
  elapsed_ms: 1,
  total: 1,
});

it('keeps complete results visible while replacing them and rejects a late response', async () => {
  const resolve: Record<string, (value: SearchResponse) => void> = {};
  vi.mocked(api.search).mockImplementation(
    (query) =>
      new Promise((done) => {
        resolve[query] = done;
      }),
  );
  const { result, rerender } = renderHook(({ query }) => useSearch(query, 'all', 0), {
    initialProps: { query: 'fire' },
  });
  await waitFor(() => expect(resolve.fire).toBeDefined());
  await act(async () => {
    resolve.fire(response('Firefox'));
  });
  rerender({ query: 'files' });
  expect(result.current.data?.results[0].name).toBe('Firefox');
  expect(result.current.pending).toBe(true);
  await waitFor(() => expect(resolve.files).toBeDefined());
  rerender({ query: 'docs' });
  await waitFor(() => expect(resolve.docs).toBeDefined());
  await act(async () => {
    resolve.docs(response('Documents'));
  });
  expect(result.current.data?.results[0].name).toBe('Documents');
  await act(async () => {
    resolve.files(response('Files'));
  });
  expect(result.current.data?.results[0].name).toBe('Documents');
});

it('does not search or show suggestions for an empty query', async () => {
  const search = vi.mocked(api.search);
  const { result } = renderHook(() => useSearch('   ', 'all', 0));
  await act(async () => await new Promise((resolve) => setTimeout(resolve, 60)));
  expect(search).not.toHaveBeenCalled();
  expect(result.current.data).toBeUndefined();
  expect(result.current.pending).toBe(false);
});

it('exposes query failures without leaving an endless loading state', async () => {
  vi.mocked(api.search).mockRejectedValueOnce(new Error('Search unavailable'));
  const { result } = renderHook(() => useSearch('test', 'file', 0));
  await waitFor(() => expect(result.current.error).toContain('Search unavailable'));
  expect(result.current.pending).toBe(false);
});
