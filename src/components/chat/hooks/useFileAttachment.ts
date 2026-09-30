import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AttachmentMeta } from "@/stores/chatStore";

/**
 * Session attachments live on `SessionSettings`, so they persist across turns
 * and reloads. This hook only picks files and reports the updated list; the
 * caller writes it back through session settings.
 */
export const useFileAttachment = (
  attachments: AttachmentMeta[],
  onChange: (attachments: AttachmentMeta[]) => void,
) => {
  const pickAttachments = useCallback(async () => {
    const picked = await invoke<AttachmentMeta[]>("pick_attachments");
    if (picked.length > 0) {
      const existing = new Set(attachments.map((a) => a.path));
      const fresh = picked.filter((a) => !existing.has(a.path));
      if (fresh.length > 0) onChange([...attachments, ...fresh]);
    }
  }, [attachments, onChange]);

  const removeAttachment = useCallback(
    (index: number) => {
      onChange(attachments.filter((_, i) => i !== index));
    },
    [attachments, onChange],
  );

  return { pickAttachments, removeAttachment };
};
