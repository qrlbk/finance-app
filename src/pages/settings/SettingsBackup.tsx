import { Download, Upload, Trash2, ExternalLink } from "lucide-react";

interface SettingsBackupProps {
  error: string | null;
  backupPath: string | null;
  backupExporting: boolean;
  onBackupExport: () => void;
  onRestoreClick: () => void;
  onResetClick: () => void;
  openContainingFolder: (path: string) => void;
}

export function SettingsBackup({
  error,
  backupPath,
  backupExporting,
  onBackupExport,
  onRestoreClick,
  onResetClick,
  openContainingFolder,
}: SettingsBackupProps) {
  return (
    <div>
      <h3 className="text-lg font-medium mb-4">Резервная копия</h3>
      <p className="text-sm text-zinc-400 mb-4">
        Создаёт копию базы данных в папке приложения. Используйте для бэкапа перед обновлениями.
      </p>
      {error && (
        <div className="p-4 rounded-lg bg-red-500/10 text-red-500 border border-red-500/20 mb-4">
          {error}
        </div>
      )}
      {backupPath && (
        <div className="p-4 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
          <div className="flex items-center justify-between gap-4">
            <span className="text-sm break-all">Копия сохранена: {backupPath}</span>
            <button
              onClick={() => openContainingFolder(backupPath)}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 btn-transition text-sm shrink-0"
            >
              <ExternalLink size={14} />
              Открыть папку
            </button>
          </div>
        </div>
      )}
      <div className="flex gap-2">
        <button
          type="button"
          onClick={onBackupExport}
          disabled={backupExporting}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Download size={18} />
          {backupExporting ? "Создаём…" : "Создать резервную копию"}
        </button>
        <button
          type="button"
          onClick={onRestoreClick}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-zinc-600 text-white hover:bg-zinc-500 transition-colors"
        >
          <Upload size={18} />
          Восстановить из копии
        </button>
        <button
          type="button"
          onClick={onResetClick}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-600 text-white hover:bg-red-700 transition-colors"
        >
          <Trash2 size={18} />
          Сбросить базу данных
        </button>
      </div>
    </div>
  );
}
