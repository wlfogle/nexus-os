import type { ReactNode } from "react";

interface ModalProps {
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
}

/**
 * Minimal modal dialog. Used for the "Fan Details" and "TLP Stats" detail
 * views - the native, in-app equivalent of the old app's "Fan GUI"/"TLP
 * Stats" buttons, which spawned an external terminal or a Qt dialog to show
 * the same kind of live/point-in-time detail. A modal here needs no
 * external process at all.
 */
export function Modal({ title, onClose, children, footer }: ModalProps) {
  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal-box" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{title}</h3>
          <button className="modal-close" onClick={onClose} aria-label="Close">
            ✕
          </button>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-footer">{footer}</div>}
      </div>
    </div>
  );
}
