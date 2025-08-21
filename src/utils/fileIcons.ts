import {
  FileText,
  Image,
  Video,
  Music,
  Archive,
  Code,
  File,
  Database,
  FileSpreadsheet,
  Settings
} from 'lucide-react';

export const getFileIcon = (fileName: string, contentType?: string) => {
  const extension = fileName.split('.').pop()?.toLowerCase();
  const mimeType = contentType?.toLowerCase();

  // Images
  if (mimeType?.startsWith('image/') || 
      ['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp', 'bmp', 'ico', 'tiff'].includes(extension || '')) {
    return Image;
  }

  // Videos
  if (mimeType?.startsWith('video/') || 
      ['mp4', 'avi', 'mov', 'mkv', 'webm', 'flv', 'wmv', 'm4v'].includes(extension || '')) {
    return Video;
  }

  // Audio
  if (mimeType?.startsWith('audio/') || 
      ['mp3', 'wav', 'flac', 'ogg', 'aac', 'm4a', 'wma'].includes(extension || '')) {
    return Music;
  }

  // Archives
  if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'lzma'].includes(extension || '')) {
    return Archive;
  }

  // Code files
  if (['js', 'ts', 'jsx', 'tsx', 'py', 'java', 'cpp', 'c', 'h', 'cs', 'php', 'rb', 'go', 'rs', 'swift', 'kt'].includes(extension || '')) {
    return Code;
  }

  // Web files
  if (['html', 'htm', 'css', 'scss', 'sass', 'less'].includes(extension || '')) {
    return Code;
  }

  // Config files
  if (['json', 'xml', 'yaml', 'yml', 'toml', 'ini', 'cfg', 'conf'].includes(extension || '')) {
    return Settings;
  }

  // Documents
  if (['pdf', 'doc', 'docx', 'txt', 'rtf', 'odt'].includes(extension || '')) {
    return FileText;
  }

  // Spreadsheets
  if (['xls', 'xlsx', 'csv', 'ods'].includes(extension || '')) {
    return FileSpreadsheet;
  }

  // Databases
  if (['db', 'sqlite', 'sql', 'mdb'].includes(extension || '')) {
    return Database;
  }

  // Default
  return File;
};