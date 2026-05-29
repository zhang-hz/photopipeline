export interface MetadataInfo {
  make?: string;
  model?: string;
  lens_model?: string;
  date_time_original?: string;
  exposure_time?: string;
  f_number?: string;
  iso?: number;
  focal_length?: string;
  latitude?: number;
  longitude?: number;
  altitude?: number;
}

export interface ImageInfo {
  id: string;
  path: string;
  filename: string;
  format: string;
  width: number;
  height: number;
  file_size_bytes: number;
  pixel_format: string;
  color_space: string;
  metadata?: MetadataInfo;
}
