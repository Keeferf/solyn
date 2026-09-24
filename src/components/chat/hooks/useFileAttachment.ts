import { useState, useRef } from "react";

export const useFileAttachment = () => {
  const [isAttachmentEnabled, setIsAttachmentEnabled] = useState(false);
  const [attachments, setAttachments] = useState<File[]>([]);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleAttachmentClick = () => {
    fileInputRef.current?.click();
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      const newFiles = Array.from(files);
      setAttachments((prev) => [...prev, ...newFiles]);

      setIsAttachmentEnabled(true);

      e.target.value = "";
    }
  };

  const removeAttachment = (index: number) => {
    setAttachments((prev) => prev.filter((_, i) => i !== index));
    if (attachments.length <= 1) {
      setIsAttachmentEnabled(false);
    }
  };

  const clearAttachments = () => {
    setAttachments([]);
    setIsAttachmentEnabled(false);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  const resetAttachment = () => {
    setAttachments([]);
    setIsAttachmentEnabled(false);
    if (fileInputRef.current) {
      fileInputRef.current.value = "";
    }
  };

  return {
    isAttachmentEnabled,
    attachments,
    fileInputRef,
    handleAttachmentClick,
    handleFileChange,
    removeAttachment,
    clearAttachments,
    resetAttachment,
  };
};
