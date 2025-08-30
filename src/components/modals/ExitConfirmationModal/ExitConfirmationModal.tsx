import React from 'react';
import { AlertTriangle } from 'lucide-react';
import { Modal, Button } from '../../ui';

interface InstallingInstance {
  name: string;
  installProgress: number;
}

interface ExitConfirmationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirmExit: () => void;
  installingInstances: InstallingInstance[];
}

const ExitConfirmationModal: React.FC<ExitConfirmationModalProps> = ({
  isOpen,
  onClose,
  onConfirmExit,
  installingInstances
}) => {
  return (
    <Modal 
      isOpen={isOpen} 
      onClose={onClose} 
      title="Confirm Exit"
      size="md"
      showCloseButton={false}
    >
      <div className="p-6">
        <div className="flex items-center gap-3 mb-4">
          <AlertTriangle className="text-yellow-400" size={24} />
          <h3 className="text-lg font-medium text-white">
            Installations in Progress
          </h3>
        </div>

        <p className="text-primary-300 mb-4">
          The following instances are still being installed. Closing the launcher now will cancel these installations:
        </p>

        <div className="space-y-2 mb-6">
          {installingInstances.map((instance, index) => (
            <div 
              key={index}
              className="p-3 bg-primary-800 rounded-lg flex items-center justify-between"
            >
              <span className="text-white font-medium">{instance.name}</span>
              <span className="text-secondary-400 text-sm">
                {Math.round(instance.installProgress)}%
              </span>
            </div>
          ))}
        </div>

        <div className="flex gap-3 justify-end">
          <Button variant="ghost" onClick={onClose}>
            Keep Running
          </Button>
          <Button variant="danger" onClick={onConfirmExit}>
            Exit Anyway
          </Button>
        </div>
      </div>
    </Modal>
  );
};

export default ExitConfirmationModal;