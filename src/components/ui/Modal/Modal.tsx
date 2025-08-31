import React from 'react';
import { X } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import * as Dialog from '@radix-ui/react-dialog';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  showCloseButton?: boolean;
}

const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  size = 'md',
  showCloseButton = true
}) => {
  const sizeClasses = {
    sm: 'max-w-md',
    md: 'max-w-2xl', 
    lg: 'max-w-4xl',
    xl: 'max-w-6xl'
  };

  return (
    <AnimatePresence>
      {isOpen && (
        <Dialog.Root open={isOpen} onOpenChange={onClose}>
          <Dialog.Portal>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="fixed inset-0 z-40"
            >
              <Dialog.Overlay className="fixed inset-0 bg-black/60 backdrop-blur-sm" />
            </motion.div>
            <Dialog.Content className="fixed inset-0 z-50 flex items-center justify-center p-4">
              <motion.div
                initial={{ opacity: 0, scale: 0.95, y: -20 }}
                animate={{ opacity: 1, scale: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95, y: -20 }}
                transition={{ 
                  duration: 0.2, 
                  ease: "easeOut",
                  type: "spring",
                  stiffness: 300,
                  damping: 25
                }}
                className={`relative bg-primary-900/95 backdrop-blur-md rounded-xl border border-primary-700/50 shadow-2xl shadow-black/50 w-full mx-auto ${sizeClasses[size]} max-h-[85vh] overflow-hidden`}
              >
                <div className="flex items-center justify-between p-6 border-b border-primary-700/50">
                  <Dialog.Title className="text-xl font-semibold text-white">
                    {title}
                  </Dialog.Title>
                  {showCloseButton && (
                    <Dialog.Close className="text-primary-400 hover:text-white transition-colors duration-200 p-1 rounded-lg hover:bg-primary-800">
                      <X size={20} />
                    </Dialog.Close>
                  )}
                </div>
                <div className="overflow-y-auto" style={{ maxHeight: 'calc(85vh - 80px)' }}>
                  {children}
                </div>
              </motion.div>
            </Dialog.Content>
          </Dialog.Portal>
        </Dialog.Root>
      )}
    </AnimatePresence>
  );
};

export default Modal;