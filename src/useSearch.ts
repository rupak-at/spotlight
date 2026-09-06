import { useEffect, useState } from 'react';
import { api } from './api';
import type { Filter, SearchResponse } from './types';

export function useSearch(query: string, filter: Filter, revision: number) {
  const key = `${revision}:${filter}:${query}`;
  const enabled = query.trim().length > 0;
  const [reply, setReply] = useState<{ key: string; data: SearchResponse }>();
  const [failure, setFailure] = useState<{ key: string; message: string }>();
  useEffect(() => {
    if (!enabled) return;
    let active = true;
    const timer = setTimeout(() => {
      void api
        .search(query, filter)
        .then((data) => {
          if (active) {
            setReply({ key, data });
            setFailure(undefined);
          }
        })
        .catch((error) => {
          if (active) setFailure({ key, message: String(error) });
        });
    }, 45);
    return () => {
      active = false;
      clearTimeout(timer);
    };
  }, [query, filter, revision, key, enabled]);
  return {
    // Keep the last complete result set visible until its replacement arrives.
    // `pending` prevents stale results from being launched in the meantime.
    data: enabled ? reply?.data : undefined,
    error: enabled && failure?.key === key ? failure.message : undefined,
    pending: enabled && reply?.key !== key && failure?.key !== key,
  };
}
