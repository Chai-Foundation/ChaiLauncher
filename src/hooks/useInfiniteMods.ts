import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ModInfo } from '../types/mods';

interface UseInfiniteModsParams {
  searchQuery?: string;
  gameVersion?: string;
  selectedCategory?: string;
  limit?: number;
}

interface UseInfiniteModsReturn {
  mods: ModInfo[];
  loading: boolean;
  hasMore: boolean;
  error: string | null;
  loadMore: () => void;
  refresh: () => void;
}

export const useInfiniteMods = ({
  searchQuery = '',
  gameVersion,
  selectedCategory,
  limit = 20
}: UseInfiniteModsParams): UseInfiniteModsReturn => {
  const [mods, setMods] = useState<ModInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentOffset, setCurrentOffset] = useState(0);

  const fetchModsPage = useCallback(async (offset: number, isSearch: boolean): Promise<ModInfo[]> => {
    try {
      let results: ModInfo[];
      
      if (isSearch && searchQuery.trim()) {
        results = await invoke<ModInfo[]>('search_mods', {
          query: searchQuery,
          gameVersion,
          modLoader: null,
          limit,
          offset
        });
      } else if (!isSearch) {
        results = await invoke<ModInfo[]>('get_featured_mods', {
          gameVersion,
          modLoader: null,
          limit,
          offset,
          category: selectedCategory || null
        });
      } else {
        return [];
      }
      
      if (results.length === 0) {
        setHasMore(false);
        return [];
      }

      if (results.length < limit) {
        setHasMore(false);
      }

      return results;
    } catch (err) {
      console.error('Failed to fetch mods page:', err);
      throw err;
    }
  }, [searchQuery, gameVersion, selectedCategory, limit]);

  const loadMore = useCallback(async () => {
    if (loading || !hasMore) return;

    setLoading(true);
    setError(null);
    
    try {
      const isSearch = searchQuery.trim().length > 0;
      const newMods = await fetchModsPage(currentOffset, isSearch);
      
      if (newMods.length > 0) {
        setMods(prev => [...prev, ...newMods]);
        setCurrentOffset(prev => prev + limit);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more mods');
    } finally {
      setLoading(false);
    }
  }, [loading, hasMore, currentOffset, fetchModsPage, limit]);

  const refresh = useCallback(async () => {
    setMods([]);
    setCurrentOffset(0);
    setHasMore(true);
    setError(null);
    setLoading(true);

    try {
      const isSearch = searchQuery.trim().length > 0;
      const initialMods = await fetchModsPage(0, isSearch);
      setMods(initialMods);
      setCurrentOffset(limit);
      
      if (initialMods.length < limit) {
        setHasMore(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load mods');
      setMods([]);
      setHasMore(false);
    } finally {
      setLoading(false);
    }
  }, [fetchModsPage, limit]);

  // Trigger refresh when search parameters change
  useEffect(() => {
    refresh();
  }, [searchQuery, gameVersion, selectedCategory]); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    mods,
    loading,
    hasMore,
    error,
    loadMore,
    refresh,
  };
};