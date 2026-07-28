// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { useEffect, useRef, useState, type ReactNode } from "react";
import { cn } from "@/lib/utils";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  children: ReactNode;
  className?: string;
}

const ANIMATION_DURATION = 200; // 与 CSS --duration-normal 保持一致

/** 模态对话框：遮罩 + 居中卡片，ESC 关闭，点击遮罩关闭。支持进出动画。
 *  复制自主应用 ui/Modal（既定组件复用策略）。 */
export function Modal({ open, onClose, children, className }: ModalProps) {
  const [visible, setVisible] = useState(false);
  const [closing, setClosing] = useState(false);
  const visibleRef = useRef(visible);
  visibleRef.current = visible;

  useEffect(() => {
    if (open) {
      setVisible(true);
      setClosing(false);
    } else if (visibleRef.current) {
      setClosing(true);
      const timer = setTimeout(() => {
        setVisible(false);
        setClosing(false);
      }, ANIMATION_DURATION);
      return () => clearTimeout(timer);
    }
  }, [open]);

  useEffect(() => {
    if (!visible) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [visible, onClose]);

  // 初始焦点落入对话框（子元素已 autoFocus 时不抢占），保证键盘可达
  const cardRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    if (!visible) return;
    const node = cardRef.current;
    if (node && !node.contains(document.activeElement)) node.focus();
  }, [visible]);

  if (!visible) return null;

  return (
    <div
      className={cn(
        "fixed inset-0 z-50 flex items-center justify-center bg-black/40",
        closing
          ? "animate-[fadeOut_var(--duration-normal)_forwards]"
          : "animate-[fadeIn_var(--duration-normal)]",
      )}
      onMouseDown={onClose}
    >
      <div
        ref={cardRef}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
        className={cn(
          "w-[min(90vw,420px)] rounded-xl bg-card p-6 outline-none [box-shadow:var(--shadow-dialog)]",
          closing
            ? "animate-[dialogExit_var(--duration-normal)_forwards]"
            : "animate-[dialogEnter_var(--duration-normal)]",
          className,
        )}
        onMouseDown={(e) => e.stopPropagation()}
      >
        {children}
      </div>
    </div>
  );
}
