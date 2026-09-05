import { useEffect, useState } from 'react';
import {
  AppWindow,
  File,
  FileArchive,
  FileAudio,
  FileCode2,
  FileImage,
  FileSpreadsheet,
  FileText,
  FileVideo,
  Folder,
  Globe,
  Terminal,
  Code2,
} from 'lucide-react';
import { api } from './api';
import type { Entry } from './types';

export function ResultIcon({ entry, revision }: { entry: Entry; revision: number }) {
  const [loaded, setLoaded] = useState<{ id: string; source: string }>();
  useEffect(() => {
    let active = true;
    void api
      .icon(entry.id)
      .then((source) => {
        if (active && source) setLoaded({ id: entry.id, source });
      })
      .catch(() => {
        /* Keep a type-specific fallback if the system theme has no icon. */
      });
    return () => {
      active = false;
    };
  }, [entry.id, revision]);
  const extension = entry.name.split('.').pop()?.toLowerCase() ?? '';
  let Icon = File;
  let category = 'document';
  if (entry.kind === 'app') {
    Icon = /firefox|browser/i.test(entry.name)
      ? Globe
      : /terminal/i.test(entry.name)
        ? Terminal
        : /code/i.test(entry.name)
          ? Code2
          : /files/i.test(entry.name)
            ? Folder
            : AppWindow;
    category = 'application';
  } else if (entry.kind === 'folder') {
    Icon = Folder;
    category = 'directory';
  } else if (/^(png|jpe?g|gif|webp|svg|avif|bmp|ico|heic)$/.test(extension)) {
    Icon = FileImage;
    category = 'image';
  } else if (/^(mp3|wav|ogg|flac|m4a|aac)$/.test(extension)) {
    Icon = FileAudio;
    category = 'audio';
  } else if (/^(mp4|mkv|webm|mov|avi)$/.test(extension)) {
    Icon = FileVideo;
    category = 'video';
  } else if (/^(zip|gz|tar|xz|7z|rar|deb)$/.test(extension)) {
    Icon = FileArchive;
    category = 'archive';
  } else if (/^(rs|js|jsx|ts|tsx|py|json|html|css|toml|yaml|yml|sh|go|c|cpp)$/.test(extension)) {
    Icon = FileCode2;
    category = 'code';
  } else if (/^(csv|xls|xlsx|ods)$/.test(extension)) {
    Icon = FileSpreadsheet;
    category = 'spreadsheet';
  } else if (/^(pdf|md|txt|doc|docx|odt|rtf)$/.test(extension)) {
    Icon = FileText;
    category = extension === 'pdf' ? 'pdf' : 'document';
  }
  return (
    <span className={`result-icon ${category}`} aria-hidden="true">
      {loaded?.id === entry.id ? (
        <img
          src={loaded.source}
          alt=""
          width="32"
          height="32"
          draggable={false}
          onError={() => setLoaded(undefined)}
        />
      ) : (
        <Icon size={23} strokeWidth={1.6} />
      )}
    </span>
  );
}
