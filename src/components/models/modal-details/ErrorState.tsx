import { ModalHeader } from "./ModalHeader";

export const ErrorState = ({
  error,
  onClose,
}: {
  error: string;
  onClose: () => void;
}) => (
  <div
    className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm animate-fadeIn"
    onClick={(e) => {
      if (e.target === e.currentTarget) onClose();
    }}
  >
    <div className="bg-[#1a1a1a] border border-white/10 rounded-2xl max-w-3xl w-full max-h-[90vh] overflow-hidden shadow-2xl animate-slideUp">
      <ModalHeader title="Error" onClose={onClose} />
      <div className="p-6">
        <div className="p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
          <p className="text-red-400 text-sm">{error}</p>
        </div>
      </div>
      <div className="p-4 border-t border-white/10 flex justify-end">
        <button
          onClick={onClose}
          className="px-4 py-2 bg-black hover:bg-white/10 rounded-lg text-white/60 hover:text-white transition-all text-sm cursor-pointer"
        >
          Close
        </button>
      </div>
    </div>
  </div>
);
