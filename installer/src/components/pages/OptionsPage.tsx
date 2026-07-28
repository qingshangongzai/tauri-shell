// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { Button } from "@/components/ui/Button";
import { Switch } from "@/components/ui/Switch";
import {
  FILE_ASSOC_DESCRIPTION,
  FILE_ASSOC_ENABLED,
  FILE_ASSOC_LABEL,
  PRODUCT_NAME,
} from "@/product";

interface OptionsPageProps {
  desktopShortcut: boolean;
  fileAssoc: boolean;
  onDesktopShortcutChange: (value: boolean) => void;
  onFileAssocChange: (value: boolean) => void;
  onBack: () => void;
  onNext: () => void;
}

/** 第三步：安装选项。桌面快捷方式 + 文件关联（可选功能，默认隐藏）。 */
export function OptionsPage({
  desktopShortcut,
  fileAssoc,
  onDesktopShortcutChange,
  onFileAssocChange,
  onBack,
  onNext,
}: OptionsPageProps) {
  return (
    <div className="flex h-full flex-col gap-8 px-12 py-10">
      <div className="flex flex-col gap-1">
        <h2 className="text-[18px] font-semibold text-text-primary">
          安装选项
        </h2>
        <p className="text-[13px] text-text-auxiliary">
          选择要一并配置的项目。
        </p>
      </div>

      <div className="flex flex-col gap-6">
        <label className="flex items-start justify-between gap-4">
          <span className="flex flex-col gap-1">
            <span className="text-[13px] text-text-body">创建桌面快捷方式</span>
            <span className="text-[12px] text-text-auxiliary">
              在桌面上放置 {PRODUCT_NAME} 的启动图标。
            </span>
          </span>
          <Switch
            checked={desktopShortcut}
            onChange={onDesktopShortcutChange}
            ariaLabel="创建桌面快捷方式"
          />
        </label>

        {FILE_ASSOC_ENABLED && (
          <label className="flex items-start justify-between gap-4">
            <span className="flex flex-col gap-1">
              <span className="text-[13px] text-text-body">
                {FILE_ASSOC_LABEL}
              </span>
              <span className="text-[12px] text-text-auxiliary">
                {FILE_ASSOC_DESCRIPTION}
              </span>
            </span>
            <Switch
              checked={fileAssoc}
              onChange={onFileAssocChange}
              ariaLabel={FILE_ASSOC_LABEL}
            />
          </label>
        )}
      </div>

      <div className="mt-auto flex justify-between">
        <Button variant="ghost" onClick={onBack}>
          上一步
        </Button>
        <Button variant="primary" onClick={onNext} className="px-8">
          安装
        </Button>
      </div>
    </div>
  );
}
