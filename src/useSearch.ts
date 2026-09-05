import { useEffect, useState } from 'react';
import { api } from './api';
import type { Filter, SearchResponse } from './types';

export function useSearch(query: string, filter: Filter, revision: number) {
  const key = `${revision}:${filter}:${query}`;
  const [reply, setReply] = useState<{ key: string; data: SearchResponse }>();
  const [failure, setFailure] = useState<{ key: string; message: string }>();
  useEffect(() => {
    let active = true;
    const timer = setTimeout(() => {
      void api.search(query, filter).then(data => {
        if (active) { setReply({ key, data }); setFailure(undefined); }
      }).catch(error => {
        if (active) setFailure({ key, message: String(error) });
      });
    }, 45);
    return () => { active = false; clearTimeout(timer); };
  }, [query, filter, revision, key]);
  return {
    data: reply?.key === key ? reply.data : undefined,
    error: failure?.key === key ? failure.message : undefined,
    pending: reply?.key !== key && failure?.key !== key,
  };
}
