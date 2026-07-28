// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { Modal } from "@/components/ui/Modal";
import { Button } from "@/components/ui/Button";
import { PRODUCT_NAME } from "@/product";

interface CloseAppConfirmDialogProps {
  open: boolean;
  /** 确认按钮文案（"继续安装" / "继续卸载"） */
  confirmLabel: string;
  /** 说明文案（区分安装/卸载语境） */
  description: string;
  onConfirm: () => void;
  onCancel: () => void;
}

/** 检测到主程序运行中的确认关闭对话框（安装/卸载共用，
 *  风格同主应用 ConfirmDialog：Modal 卡片 + ghost/primary 按钮组）。 */
export function CloseAppConfirmDialog({
  open,
  confirmLabel,
  description,
  onConfirm,
  onCancel,
}: CloseAppConfirmDialogProps) {
  return (
    <Modal open={open} onClose={onCancel}>
      <h2 className="mb-2 text-base font-semibold text-text-primary">
        {PRODUCT_NAME} 正在运行
      </h2>
      <p className="mb-4 text-sm text-text-auxiliary">{description}</p>
      <div className="flex justify-end gap-2">
        <Button type="button" variant="ghost" onClick={onCancel}>
          取消
        </Button>
        <Button type="button" variant="primary" onClick={onConfirm}>
          {confirmLabel}
        </Button>
      </div>
    </Modal>
  );
}
