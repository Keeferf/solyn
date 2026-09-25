import { Loader } from "lucide-react";
import { ModalHeader } from "./ModalHeader";

export const LoadingState = ({ onClose }: { onClose: () => void }) => (
  <div
    className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fadeIn"
    onClick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
  >
    <div className="bg-[#1a1a1a] border border-white/10 rounded-2xl max-w-3xl w-full max-h-[90vh] overflow-hidden shadow-2xl animate-slideUp">
      <ModalHeader title="Loading model details..." onClose={onClose} />
      <div className="flex items-center justify-center py-16">
        <Loader className="animate-spin text-purple-accent" size={40} />
      </div>
    </div>
  </div>
);
