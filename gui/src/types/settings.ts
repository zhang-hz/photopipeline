export interface AppSettings {
  theme: string;
  language: string;
  maxRecentFiles: number;
  checkUpdates: boolean;
  telemetry: boolean;
  serverPath: string;
  port: number;
  autoStart: boolean;
  gpuBackend: string;
  logLevel: string;
  defaultFormat: string;
  defaultDirectory: string;
  jpegQuality: number;
  embedMetadata: boolean;
  thumbnailSize: number;
  tileSize: number;
  cacheDirectory: string;
  maxCacheSize: number;
  exifToolPath: string;
}
