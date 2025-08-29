import React from 'react';
import { AlertTriangle, X } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import * as Dialog from '@radix-ui/react-dialog';

interface ExitConfirmationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirmExit: () => void;
  installingInstances: Array<{
    name: string;
    installProgress: number;
  }>;
}

const ExitConfirmationModal: React.FC<ExitConfirmationModalProps> = ({
  isOpen,
  onClose,
  onConfirmExit,
  installingInstances,
}) => {
  return (
    <Dialog.Root open={isOpen} onOpenChange={onClose}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black bg-opacity-60 z-50" />
        <Dialog.Content className="fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-primary-900/95 rounded-lg border border-primary-700 w-full max-w-md max-h-[90vh] overflow-hidden z-50 shadow-2xl">
          <div className="flex items-center justify-between p-6 border-b border-primary-700">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-secondary-500/20 rounded-lg">
                <AlertTriangle size={24} className="text-secondary-500" />
              </div>
              <h2 className="text-xl font-semibold text-white">Exit Confirmation</h2>
            </div>
            <Dialog.Close className="text-primary-400 hover:text-white transition-colors">
              <X size={20} />
            </Dialog.Close>
          </div>

          <div className="p-6">
            <div className="mb-6">
              <p className="text-primary-200 mb-4">
                {installingInstances.length === 1 
                  ? "An instance is currently being installed. Closing the launcher will interrupt the installation."
                  : `${installingInstances.length} instances are currently being installed. Closing the launcher will interrupt all installations.`
                }
              </p>
              
              <div className="space-y-3">
                {installingInstances.map((instance, index) => (
                  <div key={index} className="bg-primary-800/50 rounded-lg p-4 border border-primary-700/50">
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-white font-medium">{instance.name}</span>
                      <span className="text-primary-300 text-sm">{instance.installProgress}%</span>
                    </div>
                    <div className="w-full bg-primary-700 rounded-full h-2">
                      <motion.div
                        className="bg-gradient-to-r from-secondary-500 to-secondary-400 h-2 rounded-full"
                        style={{ width: `${instance.installProgress}%` }}
                        initial={{ width: 0 }}
                        animate={{ width: `${instance.installProgress}%` }}
                        transition={{ duration: 0.3, ease: "easeOut" }}
                      />
                    </div>
                  </div>
                ))}
              </div>
            </div>

            <div className="bg-secondary-500/10 border border-secondary-500/30 rounded-lg p-4 mb-6">
              <div className="flex items-start gap-3">
                <AlertTriangle size={20} className="text-secondary-500 flex-shrink-0 mt-0.5" />
                <div>
                  <p className="text-secondary-200 font-medium mb-1">Warning</p>
                  <p className="text-secondary-300/90 text-sm">
                    Interrupted installations may leave instances in an incomplete state and require reinstallation.
                  </p>
                </div>
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={onClose}
                className="flex-1 px-4 py-2.5 bg-primary-700 hover:bg-primary-600 text-white rounded-lg transition-colors font-medium"
              >
                Continue Installing
              </button>
              <button
                onClick={onConfirmExit}
                className="flex-1 px-4 py-2.5 bg-secondary-600 hover:bg-secondary-500 text-white rounded-lg transition-colors font-medium shadow-lg hover:shadow-secondary-500/25"
              >
                Exit Anyway
              </button>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default ExitConfirmationModal;