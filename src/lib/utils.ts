export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const k = 1024;
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const size = bytes / Math.pow(k, i);
  return `${size.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatDate(isoString: string | null): string {
  if (!isoString) return "";
  try {
    const date = new Date(isoString.replace(" ", "T") + "Z");
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return isoString;
  }
}

export function debounce<T extends (...args: never[]) => void>(
  fn: T,
  ms: number
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}

export function getExtensionColor(ext: string | null): string {
  if (!ext) return "#6b7280";
  switch (ext.toLowerCase()) {
    case "pdf":
      return "#ef4444";
    case "docx":
    case "doc":
      return "#3b82f6";
    case "js":
    case "jsx":
      return "#eab308";
    case "ts":
    case "tsx":
      return "#3b82f6";
    case "py":
      return "#22c55e";
    case "rs":
      return "#f97316";
    case "json":
      return "#a855f7";
    case "md":
      return "#06b6d4";
    case "html":
    case "htm":
      return "#f97316";
    case "css":
    case "scss":
      return "#8b5cf6";
    default:
      return "#6b7280";
  }
}
