import { ModalHeader } from "./ModalHeader";
import { RingSpinner } from "../../ui/RingSpinner";

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
        <RingSpinner
          size={40}
          className="border-[3px] border-purple-accent/25 border-t-purple-accent"
        />
      </div>
    </div>
  </div>
);
