import React from 'react';
import { Plus, Zap, TrendingUp } from 'lucide-react';
import { motion } from 'framer-motion';
import { MinecraftInstance, NewsItem } from '../types/minecraft';
import InstanceCard from './InstanceCard';

interface HomeViewProps {
  recentInstances: MinecraftInstance[];
  news: NewsItem[];
  onCreateInstance: () => void;
  onPlayInstance: (instance: MinecraftInstance) => void;
  onEditInstance: (instance: MinecraftInstance) => void;
  onDeleteInstance: (instance: MinecraftInstance) => void;
  onOpenFolder?: (instance: MinecraftInstance) => void;
}

const HomeView: React.FC<HomeViewProps> = ({
  recentInstances,
  news,
  onCreateInstance,
  onPlayInstance,
  onEditInstance,
  onDeleteInstance,
  onOpenFolder,
}) => {
  return (
    <div className="flex-1 p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold bg-gradient-to-r from-amber-200 via-white to-amber-200 bg-clip-text text-transparent mb-2">Welcome back!</h1>
        <p className="text-stone-300">Ready to dive into Minecraft?</p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <motion.button
          onClick={onCreateInstance}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          className="bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 p-6 rounded-xl text-white flex items-center gap-4 transition-all duration-300 transform hover:scale-105 shadow-lg hover:shadow-amber-500/25"
        >
          <Plus size={32} />
          <div className="text-left">
            <h3 className="font-semibold text-lg">Create Instance</h3>
            <p className="text-amber-100">Start a new adventure</p>
          </div>
        </motion.button>

        <div className="bg-stone-900/50 backdrop-blur-sm p-6 rounded-xl border border-amber-600/30 hover:border-amber-500/50 transition-all duration-300">
          <div className="flex items-center gap-3 mb-3">
            <Zap className="text-amber-400" size={24} />
            <h3 className="font-semibold text-white">Quick Launch</h3>
          </div>
          <p className="text-stone-300 text-sm">Launch your last played instance</p>
          {recentInstances[0] && (
            <button
              onClick={() => onPlayInstance(recentInstances[0])}
              className="mt-3 bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-white px-4 py-2 rounded-lg transition-all duration-300 w-full"
            >
              Play {recentInstances[0].name}
            </button>
          )}
        </div>

        <div className="bg-stone-900/50 backdrop-blur-sm p-6 rounded-xl border border-amber-600/30 hover:border-amber-500/50 transition-all duration-300">
          <div className="flex items-center gap-3 mb-3">
            <TrendingUp className="text-amber-400" size={24} />
            <h3 className="font-semibold text-white">Stats</h3>
          </div>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-stone-300">Total Instances:</span>
              <span className="text-white">{recentInstances.length}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-stone-300">Total Playtime:</span>
              <span className="text-white">
                {Math.floor(recentInstances.reduce((acc, i) => acc + i.totalPlayTime, 0) / 60)}h
              </span>
            </div>
          </div>
        </div>
      </div>

      <div>
        <h2 className="text-xl font-semibold text-white mb-4">Recent Instances</h2>
        {recentInstances.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {recentInstances.slice(0, 8).map((instance) => (
              <InstanceCard
                key={instance.id}
                instance={instance}
                onPlay={onPlayInstance}
                onEdit={onEditInstance}
                onDelete={onDeleteInstance}
                onOpenFolder={onOpenFolder}
              />
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <p className="text-stone-300 mb-4">No instances yet</p>
            <button
              onClick={onCreateInstance}
              className="bg-gradient-to-r from-amber-600 to-amber-500 hover:from-amber-500 hover:to-amber-400 text-white px-6 py-2 rounded-xl transition-all duration-300 transform hover:scale-105"
            >
              Create Your First Instance
            </button>
          </div>
        )}
      </div>

      <div>
        <h2 className="text-xl font-semibold text-white mb-4">Latest News</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {news.slice(0, 24).map((article) => (
            <motion.div
              key={article.id}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              className="bg-stone-900/50 backdrop-blur-sm rounded-xl border border-amber-600/30 overflow-hidden hover:border-amber-500/50 transition-all duration-300 hover:transform hover:scale-105"
            >
              {article.imageUrl && (
                <img
                  src={article.imageUrl}
                  alt={article.title}
                  className="w-full h-32 object-cover"
                />
              )}
              <div className="p-4">
                <h3 className="font-semibold text-white mb-2 line-clamp-2">{article.title}</h3>
                <p className="text-stone-300 text-sm mb-3 line-clamp-3">{article.summary}</p>
                <div className="flex items-center justify-between text-xs text-stone-400">
                  <span className="capitalize">{article.category}</span>
                  <span>{new Date(article.publishedAt).toLocaleDateString()}</span>
                </div>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default HomeView;