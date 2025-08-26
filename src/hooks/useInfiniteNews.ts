import { useState, useEffect, useCallback } from 'react';
import { NewsItem } from '../types/minecraft';

interface UseInfiniteNewsReturn {
  news: NewsItem[];
  loading: boolean;
  hasMore: boolean;
  error: string | null;
  loadMore: () => void;
  refresh: () => void;
}

export const useInfiniteNews = (): UseInfiniteNewsReturn => {
  const [news, setNews] = useState<NewsItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);

  const fetchNewsPage = useCallback(async (page: number): Promise<NewsItem[]> => {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const raw = await invoke('fetch_news_page', { page }) as string;
      const data = JSON.parse(raw);
      const results = data.result?.results || [];
      
      if (results.length === 0) {
        setHasMore(false);
        return [];
      }

      const mappedNews: NewsItem[] = results.map((item: any, idx: number) => ({
        id: item.url || `${page}-${idx}`,
        title: item.title,
        summary: item.description || '',
        content: '',
        publishedAt: item.time
          ? new Date(item.time * 1000).toISOString()
          : '',
        category: 'minecraft' as const,
        imageUrl: item.image || '',
        url: item.url,
      }));

      return mappedNews;
    } catch (err) {
      console.error('Failed to fetch news page:', err);
      throw err;
    }
  }, []);

  const loadMore = useCallback(async () => {
    if (loading || !hasMore) return;

    setLoading(true);
    setError(null);
    
    try {
      const newNews = await fetchNewsPage(currentPage);
      
      if (newNews.length > 0) {
        setNews(prev => [...prev, ...newNews]);
        setCurrentPage(prev => prev + 1);
      }
      
      if (newNews.length < 24) {
        setHasMore(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load more news');
    } finally {
      setLoading(false);
    }
  }, [loading, hasMore, currentPage, fetchNewsPage]);

  const refresh = useCallback(async () => {
    setNews([]);
    setCurrentPage(1);
    setHasMore(true);
    setError(null);
    setLoading(true);

    try {
      const initialNews = await fetchNewsPage(1);
      setNews(initialNews);
      setCurrentPage(2);
      
      if (initialNews.length < 24) {
        setHasMore(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load news');
      // Fallback to static news if fetch fails
      setNews([
        {
          id: '1',
          title: 'Minecraft 1.21 - The Tricky Trials Update',
          summary:
            'Brave new challenges with the Trial Chambers, fight the Breeze and Bogged, and collect their unique drops!',
          content: '',
          publishedAt: '2024-06-13T12:00:00Z',
          category: 'minecraft',
          imageUrl: '',
        },
        {
          id: '2',
          title: 'Launcher Update v2.0.0',
          summary:
            'Major launcher overhaul with improved UI, better performance, and new features.',
          content: '',
          publishedAt: '2024-01-15T12:00:00Z',
          category: 'launcher',
          imageUrl: '',
        },
      ]);
      setHasMore(false);
    } finally {
      setLoading(false);
    }
  }, [fetchNewsPage]);

  // Initial load
  useEffect(() => {
    refresh();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  return {
    news,
    loading,
    hasMore,
    error,
    loadMore,
    refresh,
  };
};