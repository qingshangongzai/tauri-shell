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
import type { UpdateInfo } from "@/lib/ipc";
import { PRODUCT_NAME } from "@/product";

interface UpdateWelcomePageProps {
  /** 检测到的已安装现场 */
  info: UpdateInfo;
  /** 本安装包携带的新版本号 */
  version: string;
  onNext: () => void;
}

/** 更新第一步：欢迎页。展示旧/新版本与安装路径 + 立即更新。 */
export function UpdateWelcomePage({
  info,
  version,
  onNext,
}: UpdateWelcomePageProps) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 px-12 text-center">
      <img src="/logo.svg" alt={PRODUCT_NAME} className="h-16 w-16" />
      <div className="flex flex-col gap-2">
        <h1 className="text-[22px] font-semibold text-text-primary">
          检测到已安装 {PRODUCT_NAME}
        </h1>
        <p className="text-[13px] text-text-auxiliary">
          {info.version ? `已安装版本 ${info.version}` : "\u00a0"}
        </p>
      </div>
      <p className="max-w-md text-[14px] leading-relaxed text-text-body">
        将更新到版本 {version}，安装位置与选项沿用上次安装。
      </p>
      <p className="max-w-md break-all text-[12px] leading-relaxed text-text-auxiliary">
        安装位置：{info.installDir}
      </p>
      <Button variant="primary" onClick={onNext} className="mt-2 px-8">
        立即更新
      </Button>
    </div>
  );
}
