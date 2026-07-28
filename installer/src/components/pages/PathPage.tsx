// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { FolderOpen } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/Button";
import { Switch } from "@/components/ui/Switch";
import { PRODUCT_NAME } from "@/product";

interface PathPageProps {
  installDir: string;
  isSystem: boolean;
  onInstallDirChange: (dir: string) => void;
  /** 切换安装范围（联动默认路径由父组件处理） */
  onSystemChange: (isSystem: boolean) => void;
  onBack: () => void;
  onNext: () => void;
}

/** 第二步：安装位置。填充式路径输入 + 浏览 + 为所有用户安装开关。 */
export function PathPage({
  installDir,
  isSystem,
  onInstallDirChange,
  onSystemChange,
  onBack,
  onNext,
}: PathPageProps) {
  const browse = async () => {
    const selected = await open({
      directory: true,
      defaultPath: installDir || undefined,
    });
    if (typeof selected === "string") onInstallDirChange(selected);
  };

  return (
    <div className="flex h-full flex-col gap-8 px-12 py-10">
      <div className="flex flex-col gap-1">
        <h2 className="text-[18px] font-semibold text-text-primary">
          选择安装位置
        </h2>
        <p className="text-[13px] text-text-auxiliary">
          {PRODUCT_NAME} 将安装到以下目录。
        </p>
      </div>

      <div className="flex flex-col gap-2">
        <label htmlFor="install-dir" className="text-[13px] text-text-body">
          安装目录
        </label>
        <div className="flex items-center gap-2">
          <input
            id="install-dir"
            type="text"
            spellCheck={false}
            value={installDir}
            onChange={(e) => onInstallDirChange(e.target.value)}
            className="min-w-0 flex-1 rounded bg-input-bg px-3 py-2 text-[13px] text-text-primary transition-colors focus:bg-input-bg-focus focus:ring-1 focus:ring-accent/20 focus:outline-none"
          />
          <Button variant="default" onClick={browse} className="shrink-0">
            <FolderOpen className="h-4 w-4" strokeWidth={1.75} />
            浏览
          </Button>
        </div>
      </div>

      <label className="flex items-start justify-between gap-4">
        <span className="flex flex-col gap-1">
          <span className="text-[13px] text-text-body">为所有用户安装</span>
          <span className="text-[12px] text-text-auxiliary">
            安装到 Program Files，供本机所有用户使用（需要管理员权限）。
          </span>
        </span>
        <Switch
          checked={isSystem}
          onChange={onSystemChange}
          ariaLabel="为所有用户安装"
        />
      </label>

      <div className="mt-auto flex justify-between">
        <Button variant="ghost" onClick={onBack}>
          上一步
        </Button>
        <Button
          variant="primary"
          onClick={onNext}
          disabled={!installDir.trim()}
          className="px-8"
        >
          下一步
        </Button>
      </div>
    </div>
  );
}
