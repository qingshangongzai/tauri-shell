// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { Trash2 } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@/components/ui/Button";
import { Switch } from "@/components/ui/Switch";
import type { UninstallInfo } from "@/lib/ipc";
import { APP_ID, PRODUCT_NAME } from "@/product";

interface UninstallConfirmPageProps {
  /** 检测到的卸载现场；null 表示未找到安装 */
  info: UninstallInfo | null;
  version: string;
  removeUserData: boolean;
  onRemoveUserDataChange: (value: boolean) => void;
  onUninstall: () => void;
}

/** 卸载第一步：确认页。展示安装信息 + 数据删除选项。 */
export function UninstallConfirmPage({
  info,
  version,
  removeUserData,
  onRemoveUserDataChange,
  onUninstall,
}: UninstallConfirmPageProps) {
  if (!info) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-6 px-12 text-center">
        <Trash2 className="h-16 w-16 text-text-placeholder" strokeWidth={1.5} />
        <div className="flex flex-col gap-2">
          <h1 className="text-[22px] font-semibold text-text-primary">
            未找到安装信息
          </h1>
          <p className="max-w-md text-[14px] leading-relaxed text-text-body">
            未在此电脑上检测到 {PRODUCT_NAME} 的安装记录，无需卸载。
          </p>
        </div>
        <Button
          variant="primary"
          onClick={() => void getCurrentWindow().close()}
        >
          关闭
        </Button>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-8 px-12 py-10">
      <div className="flex flex-col gap-1">
        <h2 className="text-[18px] font-semibold text-text-primary">
          卸载 {PRODUCT_NAME}
        </h2>
        <p className="text-[13px] text-text-auxiliary">
          将从您的电脑移除 {PRODUCT_NAME} {version}。
        </p>
      </div>

      <div className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <span className="text-[12px] text-text-auxiliary">安装位置</span>
          <span className="break-all rounded bg-sidebar px-3 py-2 text-[13px] text-text-body">
            {info.installDir}
          </span>
        </div>

        <label className="flex items-start justify-between gap-4 pt-2">
          <span className="flex flex-col gap-1">
            <span className="text-[13px] text-text-body">同时删除用户数据</span>
            <span className="text-[12px] text-text-auxiliary">
              移除 %APPDATA%\{APP_ID}
              下的全部数据与设置，删除后不可恢复。
            </span>
          </span>
          <Switch
            checked={removeUserData}
            onChange={onRemoveUserDataChange}
            ariaLabel="同时删除用户数据"
          />
        </label>
      </div>

      <div className="mt-auto flex justify-between">
        <Button variant="ghost" onClick={() => void getCurrentWindow().close()}>
          取消
        </Button>
        <Button variant="primary" onClick={onUninstall} className="px-8">
          卸载
        </Button>
      </div>
    </div>
  );
}
