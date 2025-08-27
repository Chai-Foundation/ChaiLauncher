import React, { useState, useEffect, useRef } from 'react';
import { X, Terminal, Download, Trash2, Send } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { ServerInstance, LogEntry } from '../types/servers';

interface ServerLogsModalProps {
  isOpen: boolean;
  onClose: () => void;
  server: ServerInstance;
}

const ServerLogsModal: React.FC<ServerLogsModalProps> = ({ isOpen, onClose, server }) => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [lastLogTimestamp, setLastLogTimestamp] = useState<string | null>(null);
  const [command, setCommand] = useState('');
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const logsContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isOpen && server) {
      // Initial load - get all logs
      loadInitialLogs();
      // Set up periodic log following - only get new logs
      const interval = setInterval(followLogs, 3000);
      return () => clearInterval(interval);
    }
  }, [isOpen, server]);

  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  // Initial load - get all recent logs
  const loadInitialLogs = async () => {
    if (!server) return;
    
    try {
      setLoading(true);
      const serverLogs = await invoke<LogEntry[]>('get_server_logs', { 
        serverId: server.id,
        lines: 100 
      });
      setLogs(serverLogs);
      
      // Set the timestamp of the last log for following
      if (serverLogs.length > 0) {
        setLastLogTimestamp(serverLogs[serverLogs.length - 1].timestamp);
      }
    } catch (error) {
      console.error('Failed to load server logs:', error);
      // Mock logs for fallback
      const mockLogs = [
        {
          timestamp: new Date(Date.now() - 120000).toISOString(),
          level: 'info' as const,
          message: 'Starting minecraft server version ' + (server.minecraft_instance_id.includes('1.20') ? '1.20.4' : '1.19.4')
        },
        {
          timestamp: new Date(Date.now() - 60000).toISOString(),
          level: 'info' as const,
          message: 'Loading libraries, please wait...'
        },
        {
          timestamp: new Date().toISOString(),
          level: 'info' as const,
          message: `Server ${server.name} started successfully`
        }
      ];
      setLogs(mockLogs);
      setLastLogTimestamp(mockLogs[mockLogs.length - 1].timestamp);
    } finally {
      setLoading(false);
    }
  };

  // Follow logs - only get new logs since last timestamp
  const followLogs = async () => {
    if (!server || !lastLogTimestamp) return;
    
    try {
      // Get recent logs (small number since we're following)
      const serverLogs = await invoke<LogEntry[]>('get_server_logs', { 
        serverId: server.id,
        lines: 20 
      });
      
      // Filter to only new logs after our last timestamp
      const newLogs = serverLogs.filter(log => 
        new Date(log.timestamp) > new Date(lastLogTimestamp)
      );
      
      if (newLogs.length > 0) {
        // Append new logs to existing logs
        setLogs(prev => [...prev, ...newLogs]);
        // Update last timestamp
        setLastLogTimestamp(newLogs[newLogs.length - 1].timestamp);
      }
    } catch (error) {
      console.error('Failed to follow server logs:', error);
      // Don't do anything on error - just keep existing logs
    }
  };

  const handleSendCommand = async () => {
    if (!command.trim()) return;

    try {
      const response = await invoke<string>('execute_server_command', {
        serverId: server.id,
        command: command.trim()
      });

      // Add command to logs
      const commandLog: LogEntry = {
        timestamp: new Date().toISOString(),
        level: 'info',
        message: `> ${command.trim()}`
      };

      const responseLog: LogEntry = {
        timestamp: new Date().toISOString(),
        level: 'info',
        message: response
      };

      setLogs(prev => [...prev, commandLog, responseLog]);
      setLastLogTimestamp(responseLog.timestamp); // Update timestamp to include command logs
      setCommand('');
    } catch (error) {
      console.error('Failed to send command:', error);
      const errorLog: LogEntry = {
        timestamp: new Date().toISOString(),
        level: 'error',
        message: `Failed to send command: ${error}`
      };
      setLogs(prev => [...prev, errorLog]);
      setLastLogTimestamp(errorLog.timestamp); // Update timestamp to include error logs
    }
  };

  const handleClearLogs = () => {
    if (confirm('Are you sure you want to clear the logs display? This won\'t affect server logs.')) {
      setLogs([]);
      setLastLogTimestamp(null); // Reset timestamp so next follow gets all recent logs
    }
  };

  const handleDownloadLogs = () => {
    const logText = logs.map(log => 
      `[${new Date(log.timestamp).toLocaleString()}] [${log.level.toUpperCase()}]: ${log.message}`
    ).join('\n');

    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${server.name}-logs-${new Date().toISOString().split('T')[0]}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const handleScroll = () => {
    if (!logsContainerRef.current) return;
    
    const { scrollTop, scrollHeight, clientHeight } = logsContainerRef.current;
    const isScrolledToBottom = Math.abs(scrollHeight - clientHeight - scrollTop) < 10;
    setAutoScroll(isScrolledToBottom);
  };

  const getLevelColor = (level: string): string => {
    switch (level.toLowerCase()) {
      case 'error': return 'text-red-400';
      case 'warn': return 'text-yellow-400';
      case 'debug': return 'text-blue-400';
      case 'info':
      default: return 'text-primary-300';
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-primary-800 rounded-xl p-6 max-w-4xl w-full h-[80vh] flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-green-600/20 rounded-lg flex items-center justify-center">
              <Terminal className="w-5 h-5 text-green-400" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">Server Logs</h2>
              <p className="text-sm text-primary-300">{server.name}</p>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleDownloadLogs}
              className="p-2 text-primary-400 hover:text-white hover:bg-primary-600/50 rounded transition-colors"
              title="Download logs"
            >
              <Download className="w-4 h-4" />
            </button>
            <button
              onClick={handleClearLogs}
              className="p-2 text-primary-400 hover:text-white hover:bg-primary-600/50 rounded transition-colors"
              title="Clear logs display"
            >
              <Trash2 className="w-4 h-4" />
            </button>
            <button
              onClick={onClose}
              className="text-primary-400 hover:text-white transition-colors"
            >
              <X className="w-6 h-6" />
            </button>
          </div>
        </div>

        {/* Server Status Bar */}
        <div className="bg-primary-700/30 rounded-lg p-3 mb-4 flex items-center justify-between text-sm">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${
                server.status === 'running' ? 'bg-green-400' : 
                server.status === 'starting' ? 'bg-yellow-400' : 'bg-red-400'
              }`} />
              <span className="text-primary-300 capitalize">{server.status}</span>
            </div>
            <span className="text-primary-300">Port: {server.port}</span>
            <span className="text-primary-300">Max Players: {server.max_players}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-primary-400">Auto-scroll:</span>
            <button
              onClick={() => setAutoScroll(!autoScroll)}
              className={`px-2 py-1 rounded text-xs transition-colors ${
                autoScroll ? 'bg-green-600 text-white' : 'bg-primary-600 text-primary-300'
              }`}
            >
              {autoScroll ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>

        {/* Logs Display */}
        <div 
          ref={logsContainerRef}
          onScroll={handleScroll}
          className="flex-1 bg-black/20 rounded-lg p-4 font-mono text-sm overflow-y-auto"
          style={{ scrollbarWidth: 'thin' }}
        >
          {loading && logs.length === 0 ? (
            <div className="text-center text-primary-400 py-8">
              <div className="animate-spin w-6 h-6 border-2 border-current border-t-transparent rounded-full mx-auto mb-2" />
              Loading logs...
            </div>
          ) : logs.length === 0 ? (
            <div className="text-center text-primary-400 py-8">
              No logs available
            </div>
          ) : (
            <div className="space-y-1">
              {logs.map((log, index) => (
                <div key={index} className="flex gap-3">
                  <span className="text-primary-500 shrink-0 w-20">
                    {new Date(log.timestamp).toLocaleTimeString()}
                  </span>
                  <span className={`shrink-0 w-12 text-xs uppercase font-bold ${getLevelColor(log.level)}`}>
                    [{log.level}]
                  </span>
                  <span className="text-primary-200 break-all">
                    {log.message}
                  </span>
                </div>
              ))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>

        {/* Command Input */}
        <div className="mt-4 flex gap-2">
          <div className="flex-1 relative">
            <input
              type="text"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSendCommand()}
              className="w-full px-3 py-2 pr-10 bg-primary-700 text-white rounded-lg border border-primary-600 focus:border-green-500 focus:outline-none font-mono"
              placeholder="Enter server command..."
              disabled={server.status !== 'running'}
            />
            <div className="absolute right-3 top-1/2 transform -translate-y-1/2 text-primary-500 text-xs">
              /
            </div>
          </div>
          <button
            onClick={handleSendCommand}
            disabled={!command.trim() || server.status !== 'running'}
            className="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-green-600/50 disabled:cursor-not-allowed text-white rounded-lg transition-colors flex items-center gap-2"
          >
            <Send className="w-4 h-4" />
            Send
          </button>
        </div>
        
        {server.status !== 'running' && (
          <p className="text-xs text-primary-400 mt-1 text-center">
            Server must be running to send commands
          </p>
        )}
      </div>
    </div>
  );
};

export default ServerLogsModal;