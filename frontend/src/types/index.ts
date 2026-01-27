export interface ClipboardItem {
  id: string;
  clip_type: string;
  content: string;
  preview: string;
  is_pinned: boolean;
  folder_id: string | null;
  created_at: string;
  source_app: string | null;
  source_icon: string | null;
}

export interface FolderItem {
  id: string;
  name: string;
  icon: string | null;
  color: string | null;
  is_system: boolean;
  item_count: number;
}

export interface Settings {
  max_items: number;
  auto_delete_days: number;
  startup_with_windows: boolean;
  show_in_taskbar: boolean;
  hotkey: string;
  theme: string;
}

export type ClipType = 'text' | 'image' | 'html' | 'rtf' | 'file' | 'url';

export const CLIP_TYPE_LABELS: Record<ClipType, string> = {
  text: 'Text',
  image: 'Image',
  html: 'HTML',
  rtf: 'Rich Text',
  file: 'File',
  url: 'URL',
};

export const CLIP_TYPE_ICONS: Record<ClipType, string> = {
  text: 'FileText',
  image: 'Image',
  html: 'Code',
  rtf: 'Type',
  file: 'File',
  url: 'Link',
};
