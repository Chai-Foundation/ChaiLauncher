export interface S3Config {
  endpoint: string;
  access_key: string;
  secret_key: string;
  region: string;
  bucket?: string;
  use_ssl: boolean;
  path_style: boolean;
}

export interface S3Bucket {
  name: string;
  creation_date?: string;
}

export interface S3Object {
  key: string;
  size: number;
  last_modified?: string;
  etag?: string;
  storage_class?: string;
  is_dir: boolean;
  content_type?: string;
}

export interface ObjectMetadata {
  key: string;
  size: number;
  last_modified?: string;
  etag?: string;
  content_type?: string;
  metadata: Record<string, string>;
}

export interface FileUploadProgress {
  file_name: string;
  bytes_uploaded: number;
  total_bytes: number;
  percentage: number;
}

export interface PresignedUrlRequest {
  bucket: string;
  key: string;
  expires_in: number;
  method: string;
}

export interface CopyObjectRequest {
  source_bucket: string;
  source_key: string;
  dest_bucket: string;
  dest_key: string;
}

export interface MoveObjectRequest {
  source_bucket: string;
  source_key: string;
  dest_bucket: string;
  dest_key: string;
}

export interface ConnectionProfile {
  id: string;
  name: string;
  config: S3Config;
  is_active: boolean;
  created_at: string;
  provider_type: 'aws' | 'minio' | 'digitalocean' | 'wasabi' | 'backblaze' | 'other';
}

export interface FileOperation {
  id: string;
  type: 'upload' | 'download' | 'delete' | 'copy' | 'move';
  source?: string;
  destination?: string;
  progress: number;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  error?: string;
  started_at: string;
  completed_at?: string;
}

export interface ViewMode {
  type: 'grid' | 'list' | 'tree';
  icon_size: 'small' | 'medium' | 'large';
}

export interface AppSettings {
  theme: 'amoled' | 'dark' | 'light';
  view_mode: ViewMode;
  show_hidden_files: boolean;
  auto_refresh_interval: number;
  default_download_path: string;
  keyboard_shortcuts: Record<string, string>;
}