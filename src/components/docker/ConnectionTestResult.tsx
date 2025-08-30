import React from 'react';
import { CheckCircle, X } from 'lucide-react';

interface ConnectionTestResultProps {
  testResult: { success: boolean; message: string } | null;
  testing: boolean;
}

export const ConnectionTestResult: React.FC<ConnectionTestResultProps> = ({
  testResult,
  testing
}) => {
  if (testing) {
    return (
      <div className="flex items-center gap-2 p-3 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
        <div className="animate-spin rounded-full h-4 w-4 border-2 border-yellow-500 border-t-transparent" />
        <span className="text-sm text-yellow-400">Testing connection...</span>
      </div>
    );
  }

  if (!testResult) return null;

  return (
    <div className={`flex items-center gap-2 p-3 rounded-lg ${
      testResult.success 
        ? 'bg-green-500/10 border border-green-500/30' 
        : 'bg-red-500/10 border border-red-500/30'
    }`}>
      {testResult.success ? (
        <CheckCircle className="w-4 h-4 text-green-400" />
      ) : (
        <X className="w-4 h-4 text-red-400" />
      )}
      <span className={`text-sm ${
        testResult.success ? 'text-green-400' : 'text-red-400'
      }`}>
        {testResult.message}
      </span>
    </div>
  );
};