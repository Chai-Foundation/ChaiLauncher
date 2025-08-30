import { useState, useEffect } from 'react';
import { MinecraftVersion } from '../types/minecraft';
import { MinecraftService } from '../services';

export const useMinecraftVersions = () => {
  const [minecraftVersions, setMinecraftVersions] = useState<MinecraftVersion[]>([]);
  const [versionsLoading, setVersionsLoading] = useState(true);
  const [versionsError, setVersionsError] = useState<string | null>(null);

  useEffect(() => {
    const loadVersions = async () => {
      try {
        setVersionsLoading(true);
        setVersionsError(null);
        
        const result = await MinecraftService.loadVersions();
        setMinecraftVersions(result.versions);
        
        if (result.error) {
          setVersionsError(result.error);
        }
      } catch (error) {
        console.error('Unexpected error loading versions:', error);
        setVersionsError('Unexpected error occurred while loading versions');
      } finally {
        setVersionsLoading(false);
      }
    };

    loadVersions();
  }, []);

  return {
    minecraftVersions,
    versionsLoading,
    versionsError
  };
};

export default useMinecraftVersions;