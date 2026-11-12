import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Download, Terminal, Pause, Play, Square } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import { MinecraftInstance } from '../types/minecraft';

interface LogEntry {
  timestamp: string;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

interface InstanceLogsModalProps {
  isOpen: boolean;
  onClose: () => void;
  instance?: MinecraftInstance | null;
  onStopInstance?: () => void;
}

const InstanceLogsModal: React.FC<InstanceLogsModalProps> = ({
  isOpen,
  onClose,
  instance,
  onStopInstance,
}) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isFollowing, setIsFollowing] = useState(true);
  const [autoScroll, setAutoScroll] = useState(true);
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const logsEndRef = useRef<HTMLDivElement>(null);

  // Listen for minecraft log events
  useEffect(() => {
    if (!isOpen || !instance) return;

    const unlisten = listen('minecraft_log', (event: any) => {
      const payload = event.payload;
      if (payload.instance === instance.name || payload.instance === instance.id) {
        const logEntry: LogEntry = {
          timestamp: payload.timestamp,
          level: payload.level,
          message: payload.message,
        };
        
        setLogs(prev => {
          const newLogs = [...prev, logEntry];
          // Keep only last 1000 log entries
          if (newLogs.length > 1000) {
            return newLogs.slice(-1000);
          }
          return newLogs;
        });
      }
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [isOpen, instance?.name, instance?.id]);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  // Clear logs when modal closes
  useEffect(() => {
    if (!isOpen) {
      setLogs([]);
    }
  }, [isOpen]);

  const handleScroll = () => {
    if (!logsContainerRef.current) return;
    
    const { scrollTop, scrollHeight, clientHeight } = logsContainerRef.current;
    const isScrolledToBottom = Math.abs(scrollHeight - clientHeight - scrollTop) < 10;
    setAutoScroll(isScrolledToBottom);
  };

  const handleDownloadLogs = () => {
    const logText = logs.map(log => 
      `[${new Date(log.timestamp).toLocaleString()}] [${log.level.toUpperCase()}]: ${log.message}`
    ).join('\n');

    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${instance?.name ?? 'instance'}-logs-${new Date().toISOString().split('T')[0]}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'debug': return 'text-gray-400';
      default: return 'text-green-400';
    }
  };

  if (!isOpen || !instance) return null;

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
          className="bg-primary-900 border border-primary-700 rounded-xl max-w-4xl w-full h-[80vh] flex flex-col shadow-2xl"
          onClick={e => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-primary-700">
            <div className="flex items-center gap-3">
              <Terminal className="text-secondary-400" size={20} />
              <div>
                <h2 className="text-lg font-semibold text-white">
                  {instance.name} - Console
                </h2>
                <p className="text-sm text-primary-300">
                  Minecraft {instance.version}
                </p>
              </div>
            </div>
            
            <div className="flex items-center gap-2">
              {instance.status === 'running' && onStopInstance && (
                <button
                  onClick={onStopInstance}
                  className="flex items-center gap-2 px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
                >
                  <Square size={16} />
                  Stop
                </button>
              )}
              
              <button
                onClick={() => setIsFollowing(!isFollowing)}
                className={`flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors ${
                  isFollowing 
                    ? 'bg-secondary-600 hover:bg-secondary-700 text-white' 
                    : 'bg-primary-700 hover:bg-primary-600 text-primary-300'
                }`}
              >
                {isFollowing ? <Pause size={16} /> : <Play size={16} />}
                {isFollowing ? 'Pause' : 'Resume'}
              </button>
              
              <button
                onClick={handleDownloadLogs}
                className="flex items-center gap-2 px-3 py-1.5 bg-primary-700 hover:bg-primary-600 text-primary-300 rounded-lg transition-colors"
              >
                <Download size={16} />
                Download
              </button>
              
              <button
                onClick={onClose}
                className="p-2 hover:bg-primary-700 text-primary-400 hover:text-white rounded-lg transition-colors"
              >
                <X size={20} />
              </button>
            </div>
          </div>

          {/* Logs Container */}
          <div
            ref={logsContainerRef}
            onScroll={handleScroll}
            className="flex-1 overflow-y-auto bg-gray-900 font-mono text-sm"
          >
            <div className="p-4 space-y-1">
              {logs.length === 0 ? (
                <div className="text-center text-gray-400 py-8">
                  <Terminal size={48} className="mx-auto mb-4 opacity-50" />
                  <p>Waiting for logs...</p>
                  <p className="text-xs mt-2">Logs will appear here when the instance starts</p>
                </div>
              ) : (
                logs.map((log, index) => (
                  <div key={index} className="flex gap-2 leading-relaxed">
                    <span className="text-gray-500 text-xs mt-0.5 flex-shrink-0">
                      {new Date(log.timestamp).toLocaleTimeString()}
                    </span>
                    <span className={`text-xs mt-0.5 flex-shrink-0 ${getLevelColor(log.level)}`}>
                      [{log.level.toUpperCase()}]
                    </span>
                    <span className="text-gray-300 break-all">
                      {log.message}
                    </span>
                  </div>
                ))
              )}
              <div ref={logsEndRef} />
            </div>
          </div>

          {/* Status Bar */}
          <div className="flex items-center justify-between p-3 border-t border-primary-700 bg-primary-800/50">
            <div className="flex items-center gap-4 text-xs text-primary-400">
              <span>{logs.length} log entries</span>
              {instance.status === 'running' && (
                <div className="flex items-center gap-1">
                  <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                  <span>Running</span>
                </div>
              )}
            </div>
            
            <div className="flex items-center gap-2 text-xs text-primary-400">
              <span>Auto-scroll</span>
              <button
                onClick={() => setAutoScroll(!autoScroll)}
                className={`w-8 h-4 rounded-full transition-colors relative ${
                  autoScroll ? 'bg-secondary-600' : 'bg-primary-600'
                }`}
              >
                <div
                  className={`w-3 h-3 bg-white rounded-full absolute top-0.5 transition-transform ${
                    autoScroll ? 'translate-x-4' : 'translate-x-0.5'
                  }`}
                />
              </button>
            </div>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
};

export default InstanceLogsModal;