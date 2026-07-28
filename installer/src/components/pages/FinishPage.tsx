// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { useState } from "react";
import { CheckCircle2 } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@/components/ui/Button";
import { launchApp, openDefaultAppsSettings } from "@/lib/ipc";
import { PRODUCT_NAME } from "@/product";

interface FinishPageProps {
  installDir: string;
  /** 安装时勾选了文件关联，展示"设为默认应用"引导 */
  fileAssoc: boolean;
  /** 系统级/用户级安装，深链参数据此区分 Machine/User 登记 */
  isSystem: boolean;
  /** 更新模式：文案换措辞，且不重复展示文件关联引导（登记已存在） */
  isUpdate?: boolean;
}

/** 第五步：完成页。启动主应用 + 关闭向导。 */
export function FinishPage({
  installDir,
  fileAssoc,
  isSystem,
  isUpdate = false,
}: FinishPageProps) {
  const [launchError, setLaunchError] = useState<string | null>(null);

  const launchAndClose = async () => {
    try {
      await launchApp(installDir);
      await getCurrentWindow().close();
    } catch (e) {
      setLaunchError(String(e));
    }
  };

  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 px-12 text-center">
      <CheckCircle2 className="h-16 w-16 text-accent" strokeWidth={1.5} />
      <div className="flex flex-col gap-2">
        <h1 className="text-[22px] font-semibold text-text-primary">
          {isUpdate ? "更新完成" : "安装完成"}
        </h1>
        <p className="max-w-md text-[14px] leading-relaxed text-text-body">
          {isUpdate
            ? `${PRODUCT_NAME} 已更新到最新版本。`
            : `${PRODUCT_NAME} 已成功安装到您的电脑。`}
        </p>
        {fileAssoc && !isUpdate && (
          <p className="max-w-md text-[12px] leading-relaxed text-text-auxiliary">
            {PRODUCT_NAME} 已注册为相关文件的打开方式，想设为默认应用可
            <button
              type="button"
              className="cursor-pointer text-accent hover:underline"
              onClick={() => {
                // 打不开系统设置属非关键路径，静默即可（与 launchApp 的错误兜底风格一致）
                openDefaultAppsSettings(isSystem).catch(() => {});
              }}
            >
              前往系统设置
            </button>
            。
          </p>
        )}
      </div>

      {launchError && (
        <p className="max-w-md text-[12px] text-danger">{launchError}</p>
      )}

      <div className="mt-2 flex items-center gap-3">
        <Button variant="ghost" onClick={() => void getCurrentWindow().close()}>
          完成
        </Button>
        <Button variant="primary" onClick={launchAndClose} className="px-8">
          立即启动 {PRODUCT_NAME}
        </Button>
      </div>
    </div>
  );
}
