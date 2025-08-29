import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ModrinthPack } from '../types';

interface UseInfiniteModpacksParams {
  searchQuery?: string;
  platform?: string;
  limit?: number;
}

interface UseInfiniteModpacksReturn {
  modpacks: ModrinthPack[];
  loading: boolean;
  hasMore: boolean;
  error: string | null;
  loadMore: () => void;
  refresh: () => void;
}

export const useInfiniteModpacks = ({
  searchQuery = '',
  platform = 'modrinth',
  limit = 20
}: UseInfiniteModpacksParams): UseInfiniteModpacksReturn => {
  const [modpacks, setModpacks] = useState<ModrinthPack[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentOffset, setCurrentOffset] = useState(0);

  const fetchModpacksPage = useCallback(async (offset: number): Promise<ModrinthPack[]> => {
    try {
      const query = searchQuery.trim() || 'featured';
      const results = await invoke<ModrinthPack[]>('search_modpacks', {
        query,
        platform,
        limit,
        offset
      });
      
      if (results.length === 0) {
        setHasMore(false);
        return [];
      }

      if (results.length < limit) {
        setHasMore(false);
      }

      return results;
    } catch (err) {
      console.error('Failed to fetch modpacks page:', err);
      throw err;
    }
  }, [searchQuery, platform, limit]);

  const loadMore = useCallback(async () => {
    if (loading || !hasMore) return;

    setLoading(true);
    setError(null);
    
    try {
      const newModpacks = await fetchModpacksPage(currentOffset);
      
      if (newModpacks.length > 0) {
        setModpacks(prev => [...prev, ...newModpacks]);
        setCurrentOffset(prev => prev + limit);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more modpacks');
    } finally {
      setLoading(false);
    }
  }, [loading, hasMore, currentOffset, fetchModpacksPage, limit]);

  const refresh = useCallback(async () => {
    setModpacks([]);
    setCurrentOffset(0);
    setHasMore(true);
    setError(null);
    setLoading(true);

    try {
      const initialModpacks = await fetchModpacksPage(0);
      setModpacks(initialModpacks);
      setCurrentOffset(limit);
      
      if (initialModpacks.length < limit) {
        setHasMore(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load modpacks');
      setModpacks([]);
      setHasMore(false);
    } finally {
      setLoading(false);
    }
  }, [fetchModpacksPage, limit]);

  // Trigger refresh when search parameters change
  useEffect(() => {
    refresh();
  }, [searchQuery, platform]); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    modpacks,
    loading,
    hasMore,
    error,
    loadMore,
    refresh,
  };
};