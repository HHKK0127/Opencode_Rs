import { createSignal, createResource, For, Show } from 'solid-js';
import { api } from '../services/api';
import type { FileItem } from '../types/api';

interface FileExplorerProps {
  onFileSelect: (file: FileItem) => void;
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const size = (bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0);
  return `${size} ${units[i]}`;
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  } catch {
    return dateStr;
  }
}

function getFileIcon(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase();
  const iconMap: Record<string, string> = {
    ts: '🔵', tsx: '⚛️', js: '🟡', jsx: '⚛️',
    rs: '🦀', py: '🐍', go: '🔷',
    json: '📋', yml: '⚙️', yaml: '⚙️', toml: '⚙️',
    md: '📝', txt: '📄',
    html: '🌐', css: '🎨', scss: '🎨',
    png: '🖼️', jpg: '🖼️', jpeg: '🖼️', svg: '🖼️',
    gitignore: '🙈', dockerfile: '🐳',
    sh: '💻', bat: '🪟', ps1: '🪟'
  };
  return iconMap[ext || ''] || '📄';
}

function FileExplorer(props: FileExplorerProps) {
  const [page, setPage] = createSignal(1);
  const perPage = 20;

  const [filesResource] = createResource(
    () => page(),
    async (p: number) => {
      const result = await api.files.list(p, perPage);
      if (result.error) throw new Error(result.error);
      return result.data!;
    }
  );

  const totalPages = () => {
    const data = filesResource();
    if (!data) return 1;
    return Math.max(1, Math.ceil(data.total / data.per_page));
  };

  return (
    <div style={{ padding: '1.5rem' }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        'justify-content': 'space-between',
        'align-items': 'center',
        'margin-bottom': '1rem'
      }}>
        <h2 style={{ margin: 0, 'font-size': '1.1rem', color: '#e0e0ff', 'font-weight': 500 }}>
          Files
        </h2>
        <Show when={filesResource()}>
          <span style={{ 'font-size': '0.8rem', color: '#8888aa' }}>
            {filesResource()!.total} files
          </span>
        </Show>
      </div>

      {/* Loading State */}
      <Show when={filesResource.loading}>
        <div style={{ padding: '2rem', 'text-align': 'center', color: '#8888aa' }}>
          <div style={{ 'font-size': '1.5rem', 'margin-bottom': '0.5rem' }}>⏳</div>
          <div style={{ 'font-size': '0.85rem' }}>Loading files...</div>
        </div>
      </Show>

      {/* Error State */}
      <Show when={filesResource.error}>
        <div style={{
          padding: '1rem',
          'background-color': 'rgba(255, 70, 70, 0.1)',
          border: '1px solid rgba(255, 70, 70, 0.2)',
          'border-radius': '8px',
          color: '#ff6b6b',
          'font-size': '0.85rem',
          'text-align': 'center'
        }}>
          ❌ Failed to load files: {filesResource.error.message}
        </div>
      </Show>

      {/* File List */}
      <Show when={filesResource() && !filesResource.loading}>
        <div style={{
          'background-color': 'rgba(255,255,255,0.02)',
          'border-radius': '8px',
          border: '1px solid rgba(255,255,255,0.06)',
          overflow: 'hidden'
        }}>
          {/* Table Header */}
          <div style={{
            display: 'grid',
            'grid-template-columns': 'auto 1fr 100px 160px',
            gap: '0.5rem',
            padding: '0.6rem 1rem',
            'border-bottom': '1px solid rgba(255,255,255,0.06)',
            'font-size': '0.75rem',
            color: '#666688',
            'text-transform': 'uppercase',
            'letter-spacing': '0.5px'
          }}>
            <span></span>
            <span>Name</span>
            <span style={{ 'text-align': 'right' }}>Size</span>
            <span>Updated</span>
          </div>

          {/* File Rows */}
          <For each={filesResource()!.files}>
            {(file) => (
              <div
                onClick={() => props.onFileSelect(file)}
                style={{
                  display: 'grid',
                  'grid-template-columns': 'auto 1fr 100px 160px',
                  gap: '0.5rem',
                  padding: '0.6rem 1rem',
                  'border-bottom': '1px solid rgba(255,255,255,0.03)',
                  'font-size': '0.85rem',
                  cursor: 'pointer',
                  transition: 'background-color 0.1s',
                  'align-items': 'center'
                }}
                onMouseEnter={(e) => e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.03)'}
                onMouseLeave={(e) => e.currentTarget.style.backgroundColor = 'transparent'}
              >
                <span style={{ 'font-size': '1.1rem' }}>{getFileIcon(file.filename)}</span>
                <span style={{ color: '#ccccee', overflow: 'hidden', 'text-overflow': 'ellipsis', 'white-space': 'nowrap' }}>
                  {file.filename}
                </span>
                <span style={{ 'text-align': 'right', color: '#8888aa', 'font-size': '0.8rem' }}>
                  {formatFileSize(file.size)}
                </span>
                <span style={{ color: '#7777aa', 'font-size': '0.78rem' }}>
                  {formatDate(file.updated_at)}
                </span>
              </div>
            )}
          </For>
        </div>

        {/* Pagination */}
        <Show when={totalPages() > 1}>
          <div style={{
            display: 'flex',
            'justify-content': 'center',
            'align-items': 'center',
            gap: '0.5rem',
            'margin-top': '1rem'
          }}>
            <button
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page() <= 1}
              style={{
                padding: '0.4rem 0.75rem',
                'background-color': page() <= 1 ? 'rgba(255,255,255,0.03)' : 'rgba(85, 85, 204, 0.15)',
                border: '1px solid rgba(85, 85, 204, 0.2)',
                'border-radius': '6px',
                color: page() <= 1 ? '#555577' : '#8888cc',
                cursor: page() <= 1 ? 'default' : 'pointer',
                'font-size': '0.8rem'
              }}
            >
              ← Prev
            </button>
            <span style={{ 'font-size': '0.8rem', color: '#8888aa' }}>
              Page {page()} of {totalPages()}
            </span>
            <button
              onClick={() => setPage((p) => Math.min(totalPages(), p + 1))}
              disabled={page() >= totalPages()}
              style={{
                padding: '0.4rem 0.75rem',
                'background-color': page() >= totalPages() ? 'rgba(255,255,255,0.03)' : 'rgba(85, 85, 204, 0.15)',
                border: '1px solid rgba(85, 85, 204, 0.2)',
                'border-radius': '6px',
                color: page() >= totalPages() ? '#555577' : '#8888cc',
                cursor: page() >= totalPages() ? 'default' : 'pointer',
                'font-size': '0.8rem'
              }}
            >
              Next →
            </button>
          </div>
        </Show>
      </Show>
    </div>
  );
}

export default FileExplorer;