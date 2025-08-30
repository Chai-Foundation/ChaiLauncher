import React, { useState } from 'react';
import { Folder, Calendar, Package, HardDrive, User } from 'lucide-react';
import { MinecraftInstance } from '../../types/minecraft';

interface GeneralTabProps {
  instance: MinecraftInstance;
  onSave: (updates: Partial<MinecraftInstance>) => Promise<boolean>;
}

export const GeneralTab: React.FC<GeneralTabProps> = ({ instance, onSave }) => {
  const [formData, setFormData] = useState({
    name: instance.name,
    description: instance.description || '',
    tags: instance.tags || []
  });
  const [newTag, setNewTag] = useState('');
  const [isSaving, setIsSaving] = useState(false);

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const success = await onSave({
        name: formData.name,
        description: formData.description,
        tags: formData.tags
      });
      
      if (success) {
        // Show success message or update UI
      } else {
        alert('Failed to save settings');
      }
    } finally {
      setIsSaving(false);
    }
  };

  const addTag = () => {
    const tag = newTag.trim();
    if (tag && !formData.tags.includes(tag)) {
      setFormData(prev => ({
        ...prev,
        tags: [...prev.tags, tag]
      }));
      setNewTag('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setFormData(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove)
    }));
  };

  return (
    <div className="space-y-6">
      {/* Basic Information */}
      <div className="bg-primary-700/30 p-4 rounded-lg">
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <User className="w-5 h-5" />
          Basic Information
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Instance Name
            </label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
              className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-primary-300 mb-2">
              Minecraft Version
            </label>
            <input
              type="text"
              value={instance.version}
              disabled
              className="w-full px-3 py-2 bg-primary-800 text-primary-400 rounded-lg border border-primary-600"
            />
          </div>
        </div>

        <div className="mt-4">
          <label className="block text-sm font-medium text-primary-300 mb-2">
            Description
          </label>
          <textarea
            value={formData.description}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            rows={3}
            className="w-full px-3 py-2 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-secondary-500 focus:outline-none"
            placeholder="Add a description for this instance..."
          />
        </div>

        {/* Tags */}
        <div className="mt-4">
          <label className="block text-sm font-medium text-primary-300 mb-2">
            Tags
          </label>
          <div className="flex flex-wrap gap-2 mb-2">
            {formData.tags.map((tag) => (
              <span
                key={tag}
                className="bg-secondary-600/20 text-secondary-400 px-2 py-1 rounded text-sm flex items-center gap-1"
              >
                {tag}
                <button
                  onClick={() => removeTag(tag)}
                  className="text-secondary-400 hover:text-red-400 ml-1"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
          <div className="flex gap-2">
            <input
              type="text"
              value={newTag}
              onChange={(e) => setNewTag(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && addTag()}
              className="flex-1 px-3 py-1 bg-primary-700 text-white rounded border border-primary-600 focus:border-secondary-500 focus:outline-none text-sm"
              placeholder="Add a tag..."
            />
            <button
              onClick={addTag}
              disabled={!newTag.trim()}
              className="px-3 py-1 bg-secondary-600 hover:bg-secondary-700 disabled:bg-secondary-800 text-white rounded text-sm transition-colors"
            >
              Add
            </button>
          </div>
        </div>
      </div>

      {/* Instance Information */}
      <div className="bg-primary-700/30 p-4 rounded-lg">
        <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
          <Package className="w-5 h-5" />
          Instance Information
        </h3>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div className="flex items-center gap-3">
            <Folder className="w-5 h-5 text-primary-400" />
            <div>
              <p className="text-sm text-primary-300">Game Directory</p>
              <p className="text-white text-sm font-mono break-all">{instance.gameDir}</p>
            </div>
          </div>
          
          <div className="flex items-center gap-3">
            <Calendar className="w-5 h-5 text-primary-400" />
            <div>
              <p className="text-sm text-primary-300">Last Played</p>
              <p className="text-white text-sm">
                {instance.lastPlayed ? new Date(instance.lastPlayed).toLocaleDateString() : 'Never'}
              </p>
            </div>
          </div>
          
          <div className="flex items-center gap-3">
            <HardDrive className="w-5 h-5 text-primary-400" />
            <div>
              <p className="text-sm text-primary-300">Size</p>
              <p className="text-white text-sm">
                {instance.sizeMb ? `${(instance.sizeMb / 1024).toFixed(1)} GB` : 'Calculating...'}
              </p>
            </div>
          </div>
        </div>

        {instance.isModded && (
          <div className="mt-4 p-3 bg-secondary-600/10 border border-secondary-600/30 rounded-lg">
            <p className="text-secondary-400 text-sm">
              <Package className="w-4 h-4 inline mr-2" />
              This instance has {instance.modsCount} mods installed
            </p>
          </div>
        )}
      </div>

      {/* Save Button */}
      <div className="flex justify-end">
        <button
          onClick={handleSave}
          disabled={isSaving}
          className="bg-secondary-600 hover:bg-secondary-700 disabled:bg-secondary-800 text-white px-6 py-2 rounded-lg transition-colors"
        >
          {isSaving ? 'Saving...' : 'Save Changes'}
        </button>
      </div>
    </div>
  );
};