import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion } from 'framer-motion';
import { 
  Package,
  Upload,
  Download,
  Settings,
  FileText,
  Image,
  Tag,
  User,
  Calendar,
  Loader,
  CheckCircle,
  AlertCircle,
  X,
  Info
} from 'lucide-react';
import { MinecraftInstance } from '../types';

interface ModpackCreatorProps {
  instances: MinecraftInstance[];
  onClose: () => void;
  onCreateSuccess?: (modpackPath: string) => void;
}

interface ModpackMetadata {
  name: string;
  version: string;
  author: string;
  description: string;
  minecraftVersion: string;
  tags: string[];
  iconPath?: string;
  includeUserData: boolean;
  includeResourcePacks: boolean;
  includeShaderPacks: boolean;
  includeConfig: boolean;
  includeSaves: boolean;
}

export default function ModpackCreator({ instances, onClose, onCreateSuccess }: ModpackCreatorProps) {
  const [selectedInstance, setSelectedInstance] = useState<MinecraftInstance | null>(null);
  const [metadata, setMetadata] = useState<ModpackMetadata>({
    name: '',
    version: '1.0.0',
    author: '',
    description: '',
    minecraftVersion: '',
    tags: [],
    includeUserData: false,
    includeResourcePacks: true,
    includeShaderPacks: true,
    includeConfig: true,
    includeSaves: false,
  });
  const [currentTag, setCurrentTag] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (selectedInstance && !metadata.name) {
      setMetadata(prev => ({
        ...prev,
        name: `${selectedInstance.name} Modpack`,
        minecraftVersion: selectedInstance.version,
        description: `A modpack based on the ${selectedInstance.name} instance.`
      }));
    }
  }, [selectedInstance, metadata.name]);

  const handleInstanceSelect = (instance: MinecraftInstance) => {
    setSelectedInstance(instance);
    setError(null);
  };

  const handleAddTag = () => {
    if (currentTag.trim() && !metadata.tags.includes(currentTag.trim())) {
      setMetadata(prev => ({
        ...prev,
        tags: [...prev.tags, currentTag.trim()]
      }));
      setCurrentTag('');
    }
  };

  const handleRemoveTag = (tagToRemove: string) => {
    setMetadata(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove)
    }));
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAddTag();
    }
  };

  const validateForm = (): boolean => {
    if (!selectedInstance) {
      setError('Please select an instance to create a modpack from');
      return false;
    }
    if (!metadata.name.trim()) {
      setError('Please enter a modpack name');
      return false;
    }
    if (!metadata.author.trim()) {
      setError('Please enter an author name');
      return false;
    }
    if (!metadata.description.trim()) {
      setError('Please enter a description');
      return false;
    }
    return true;
  };

  const createModpack = async () => {
    if (!validateForm()) return;

    setIsCreating(true);
    setProgress(0);
    setStatus('Initializing modpack creation...');
    setError(null);

    try {
      // This would call a Tauri command to create the modpack
      const modpackData = {
        instanceId: selectedInstance!.id,
        instancePath: selectedInstance!.gameDir,
        metadata: {
          ...metadata,
          minecraftVersion: selectedInstance!.version,
        }
      };

      // Simulate progress updates (in real implementation, this would come from backend)
      const progressSteps = [
        { progress: 10, status: 'Analyzing instance files...' },
        { progress: 25, status: 'Collecting mods and dependencies...' },
        { progress: 40, status: 'Packaging resource packs...' },
        { progress: 55, status: 'Exporting configuration files...' },
        { progress: 70, status: 'Creating modpack manifest...' },
        { progress: 85, status: 'Compressing modpack archive...' },
        { progress: 100, status: 'Modpack creation complete!' }
      ];

      for (const step of progressSteps) {
        await new Promise(resolve => setTimeout(resolve, 800));
        setProgress(step.progress);
        setStatus(step.status);
      }

      // In real implementation, this would be the actual path returned by the backend
      const modpackPath = `/modpacks/${metadata.name.replace(/\s+/g, '_')}_v${metadata.version}.zip`;
      
      setSuccess(true);
      if (onCreateSuccess) {
        onCreateSuccess(modpackPath);
      }

    } catch (error) {
      console.error('Failed to create modpack:', error);
      setError(`Failed to create modpack: ${error}`);
    } finally {
      setIsCreating(false);
    }
  };

  if (success) {
    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          className="bg-stone-800 rounded-lg p-6 w-full max-w-md mx-4"
        >
          <div className="text-center">
            <CheckCircle className="w-16 h-16 text-green-500 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">Modpack Created Successfully!</h3>
            <p className="text-gray-300 mb-6">
              Your modpack "{metadata.name}" has been created and is ready to share.
            </p>
            <div className="flex gap-3">
              <button
                onClick={onClose}
                className="flex-1 px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg transition-colors"
              >
                Done
              </button>
            </div>
          </div>
        </motion.div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <motion.div
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="bg-stone-800 rounded-lg p-6 w-full max-w-4xl mx-4 max-h-[90vh] overflow-y-auto"
      >
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <Package className="w-6 h-6 text-amber-500" />
            <h2 className="text-2xl font-semibold text-white">Create Modpack</h2>
          </div>
          <button
            onClick={onClose}
            disabled={isCreating}
            className="text-gray-400 hover:text-white disabled:opacity-50"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {error && (
          <div className="mb-6 p-4 bg-red-900/50 border border-red-500/50 rounded-lg flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0" />
            <span className="text-red-200">{error}</span>
          </div>
        )}

        {isCreating ? (
          <div className="text-center py-8">
            <Loader className="w-12 h-12 animate-spin text-amber-500 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">Creating Modpack</h3>
            <p className="text-gray-300 mb-4">{status}</p>
            <div className="w-full bg-stone-700 rounded-full h-2 mb-2">
              <div 
                className="bg-amber-500 h-2 rounded-full transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
            <span className="text-sm text-gray-400">{progress}% complete</span>
          </div>
        ) : (
          <div className="space-y-6">
            {/* Instance Selection */}
            <div>
              <label className="block text-sm font-medium text-gray-200 mb-2">
                Select Instance
              </label>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {instances.map((instance) => (
                  <button
                    key={instance.id}
                    onClick={() => handleInstanceSelect(instance)}
                    className={`p-3 rounded-lg border transition-all text-left ${
                      selectedInstance?.id === instance.id
                        ? 'border-amber-500 bg-amber-500/10'
                        : 'border-stone-600 bg-stone-700 hover:border-stone-500'
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <Package className="w-5 h-5 text-amber-500" />
                      <div>
                        <div className="font-medium text-white">{instance.name}</div>
                        <div className="text-sm text-gray-400">
                          Minecraft {instance.version} • {instance.modsCount} mods
                        </div>
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {selectedInstance && (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Basic Information */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white flex items-center gap-2">
                    <FileText className="w-5 h-5 text-amber-500" />
                    Basic Information
                  </h3>

                  <div>
                    <label className="block text-sm font-medium text-gray-200 mb-1">
                      Modpack Name *
                    </label>
                    <input
                      type="text"
                      value={metadata.name}
                      onChange={(e) => setMetadata(prev => ({ ...prev, name: e.target.value }))}
                      className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white focus:outline-none focus:border-amber-500"
                      placeholder="My Awesome Modpack"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-200 mb-1">
                      Version *
                    </label>
                    <input
                      type="text"
                      value={metadata.version}
                      onChange={(e) => setMetadata(prev => ({ ...prev, version: e.target.value }))}
                      className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white focus:outline-none focus:border-amber-500"
                      placeholder="1.0.0"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-200 mb-1">
                      Author *
                    </label>
                    <input
                      type="text"
                      value={metadata.author}
                      onChange={(e) => setMetadata(prev => ({ ...prev, author: e.target.value }))}
                      className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white focus:outline-none focus:border-amber-500"
                      placeholder="Your Name"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-200 mb-1">
                      Description *
                    </label>
                    <textarea
                      value={metadata.description}
                      onChange={(e) => setMetadata(prev => ({ ...prev, description: e.target.value }))}
                      rows={3}
                      className="w-full px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white focus:outline-none focus:border-amber-500"
                      placeholder="Describe your modpack..."
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-200 mb-1">
                      Tags
                    </label>
                    <div className="flex gap-2 mb-2">
                      <input
                        type="text"
                        value={currentTag}
                        onChange={(e) => setCurrentTag(e.target.value)}
                        onKeyPress={handleKeyPress}
                        className="flex-1 px-3 py-2 bg-stone-700 border border-stone-600 rounded-lg text-white focus:outline-none focus:border-amber-500"
                        placeholder="Add a tag..."
                      />
                      <button
                        onClick={handleAddTag}
                        className="px-4 py-2 bg-amber-600 hover:bg-amber-500 text-white rounded-lg transition-colors"
                      >
                        <Tag className="w-4 h-4" />
                      </button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {metadata.tags.map((tag) => (
                        <span
                          key={tag}
                          className="px-2 py-1 bg-amber-600/20 text-amber-200 rounded text-sm flex items-center gap-1"
                        >
                          {tag}
                          <button
                            onClick={() => handleRemoveTag(tag)}
                            className="text-amber-300 hover:text-amber-100"
                          >
                            <X className="w-3 h-3" />
                          </button>
                        </span>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Include Options */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white flex items-center gap-2">
                    <Settings className="w-5 h-5 text-amber-500" />
                    Include Options
                  </h3>

                  <div className="space-y-3">
                    <label className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={metadata.includeConfig}
                        onChange={(e) => setMetadata(prev => ({ ...prev, includeConfig: e.target.checked }))}
                        className="w-4 h-4 text-amber-600 bg-stone-700 border-stone-600 rounded focus:ring-amber-500"
                      />
                      <span className="text-white">Configuration files</span>
                      <div title="Include mod configuration and settings">
                        <Info className="w-4 h-4 text-gray-400" />
                      </div>
                    </label>

                    <label className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={metadata.includeResourcePacks}
                        onChange={(e) => setMetadata(prev => ({ ...prev, includeResourcePacks: e.target.checked }))}
                        className="w-4 h-4 text-amber-600 bg-stone-700 border-stone-600 rounded focus:ring-amber-500"
                      />
                      <span className="text-white">Resource packs</span>
                      <div title="Include installed resource packs">
                        <Info className="w-4 h-4 text-gray-400" />
                      </div>
                    </label>

                    <label className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={metadata.includeShaderPacks}
                        onChange={(e) => setMetadata(prev => ({ ...prev, includeShaderPacks: e.target.checked }))}
                        className="w-4 h-4 text-amber-600 bg-stone-700 border-stone-600 rounded focus:ring-amber-500"
                      />
                      <span className="text-white">Shader packs</span>
                      <div title="Include installed shader packs">
                        <Info className="w-4 h-4 text-gray-400" />
                      </div>
                    </label>

                    <label className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={metadata.includeSaves}
                        onChange={(e) => setMetadata(prev => ({ ...prev, includeSaves: e.target.checked }))}
                        className="w-4 h-4 text-amber-600 bg-stone-700 border-stone-600 rounded focus:ring-amber-500"
                      />
                      <span className="text-white">World saves</span>
                      <div title="Include world saves (not recommended for distribution)">
                        <Info className="w-4 h-4 text-gray-400" />
                      </div>
                    </label>

                    <label className="flex items-center gap-3">
                      <input
                        type="checkbox"
                        checked={metadata.includeUserData}
                        onChange={(e) => setMetadata(prev => ({ ...prev, includeUserData: e.target.checked }))}
                        className="w-4 h-4 text-amber-600 bg-stone-700 border-stone-600 rounded focus:ring-amber-500"
                      />
                      <span className="text-white">User data and logs</span>
                      <div title="Include user-specific data and logs (not recommended)">
                        <Info className="w-4 h-4 text-gray-400" />
                      </div>
                    </label>
                  </div>

                  <div className="mt-6 p-4 bg-stone-700/50 rounded-lg">
                    <h4 className="font-medium text-white mb-2">Modpack Summary</h4>
                    <div className="text-sm text-gray-300 space-y-1">
                      <div>Base Instance: {selectedInstance.name}</div>
                      <div>Minecraft Version: {selectedInstance.version}</div>
                      <div>Mods: {selectedInstance.modsCount}</div>
                      <div>Format: CurseForge (.zip)</div>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex justify-end gap-3 pt-4 border-t border-stone-600">
              <button
                onClick={onClose}
                disabled={isCreating}
                className="px-4 py-2 text-gray-300 hover:text-white disabled:opacity-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={createModpack}
                disabled={!selectedInstance || isCreating}
                className="flex items-center gap-2 px-6 py-2 bg-amber-600 hover:bg-amber-500 disabled:bg-stone-600 text-white rounded-lg transition-colors"
              >
                <Upload className="w-4 h-4" />
                Create Modpack
              </button>
            </div>
          </div>
        )}
      </motion.div>
    </div>
  );
}