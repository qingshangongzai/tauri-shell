// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { Minus, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cn } from "@/lib/utils";
import { PRODUCT_NAME } from "@/product";

interface TitleBarProps {
  /** 安装/卸载进行中禁用关闭，防止中断写注册表/删除文件 */
  closable: boolean;
  /** 标题文案（卸载模式为"{产品名} 卸载"） */
  title?: string;
}

/** 安装器标题栏：可拖拽区域 + logo/产品名 + 最小化/关闭（精简自主应用 TitleBar）。 */
export function TitleBar({
  closable,
  title = `${PRODUCT_NAME} 安装向导`,
}: TitleBarProps) {
  const win = getCurrentWindow();

  return (
    <div
      data-tauri-drag-region
      className="flex h-8 shrink-0 items-center justify-between bg-titlebar select-none"
    >
      <div
        data-tauri-drag-region
        className="flex items-center gap-2 px-3 text-[12px] text-text-auxiliary"
      >
        <img src="/logo.svg" alt={PRODUCT_NAME} className="h-4 w-4" />
        <span className="font-semibold text-text-primary">{title}</span>
      </div>
      <div className="flex items-center">
        <button
          type="button"
          onClick={() => void win.minimize()}
          title="最小化"
          aria-label="最小化"
          className="flex h-8 w-11 items-center justify-center text-text-auxiliary transition-colors hover:bg-hover hover:text-text-primary"
        >
          <Minus className="h-4 w-4" strokeWidth={1.75} />
        </button>
        <button
          type="button"
          onClick={() => void win.close()}
          disabled={!closable}
          title="关闭"
          aria-label="关闭"
          className={cn(
            "flex h-8 w-11 items-center justify-center text-text-auxiliary transition-colors",
            closable
              ? "hover:bg-danger hover:text-white"
              : "cursor-not-allowed opacity-40",
          )}
        >
          <X className="h-4 w-4" strokeWidth={1.75} />
        </button>
      </div>
    </div>
  );
}
