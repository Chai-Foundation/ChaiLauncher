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
    <Dialog.Root open={isOpen} onOpenChange={onClose}>
      <Dialog.Portal>
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
        >
          <Dialog.Overlay className="fixed inset-0 bg-black bg-opacity-50 z-50 modal-backdrop" />
        </motion.div>
        <Dialog.Content>
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
            className={`fixed top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 bg-primary-900/95 backdrop-blur-md rounded-lg border border-primary-700 shadow-2xl shadow-black/50 w-full ${sizeClasses[size]} max-h-[90vh] overflow-hidden z-50`}
          >
            <div className="flex items-center justify-between p-6 border-b border-primary-700">
              <Dialog.Title className="text-xl font-semibold text-white">
                {title}
              </Dialog.Title>
              {showCloseButton && (
                <Dialog.Close className="text-primary-400 hover:text-white transition-colors duration-200">
                  <X size={20} />
                </Dialog.Close>
              )}
            </div>
            <div className="overflow-y-auto" style={{ maxHeight: 'calc(90vh - 80px)' }}>
              {children}
            </div>
          </motion.div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default Modal;