import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { AlertTriangle, X } from 'lucide-react';
import { MinecraftInstance } from '../types/minecraft';

interface StopInstanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  instance: MinecraftInstance | null;
}

const StopInstanceModal: React.FC<StopInstanceModalProps> = ({
  isOpen,
  onClose,
  onConfirm,
  instance,
}) => {
  if (!isOpen || !instance) return null;

  const handleConfirm = () => {
    onConfirm();
    onClose();
  };

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
        onClick={onClose}
      >
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.9 }}
          className="bg-primary-900 border border-primary-700 rounded-xl max-w-md w-full shadow-2xl"
          onClick={e => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-primary-700">
            <div className="flex items-center gap-3">
              <AlertTriangle className="text-yellow-400" size={20} />
              <h2 className="text-lg font-semibold text-white">
                Stop Instance
              </h2>
            </div>
            <button
              onClick={onClose}
              className="p-1 hover:bg-primary-700 text-primary-400 hover:text-white rounded-lg transition-colors"
            >
              <X size={20} />
            </button>
          </div>

          {/* Content */}
          <div className="p-6">
            <p className="text-primary-300 mb-4">
              Are you sure you want to stop "{instance.name}"?
            </p>
            
            <div className="bg-yellow-900/20 border border-yellow-700/50 rounded-lg p-3 mb-6">
              <p className="text-yellow-300 text-sm">
                This will forcefully terminate the Minecraft process. Any unsaved progress in the game will be lost.
              </p>
            </div>

            <div className="flex gap-3 justify-end">
              <button
                onClick={onClose}
                className="px-4 py-2 bg-primary-700 hover:bg-primary-600 text-primary-300 hover:text-white rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
              >
                Stop Instance
              </button>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
};

export default StopInstanceModal;