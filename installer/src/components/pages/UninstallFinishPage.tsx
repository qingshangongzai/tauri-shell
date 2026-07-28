// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { CheckCircle2 } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@/components/ui/Button";
import { PRODUCT_NAME } from "@/product";

/** 卸载第三步：完成页。 */
export function UninstallFinishPage() {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 px-12 text-center">
      <CheckCircle2 className="h-16 w-16 text-accent" strokeWidth={1.5} />
      <div className="flex flex-col gap-2">
        <h1 className="text-[22px] font-semibold text-text-primary">
          卸载完成
        </h1>
        <p className="max-w-md text-[14px] leading-relaxed text-text-body">
          {PRODUCT_NAME} 已从您的电脑移除。感谢使用，期待与您再会。
        </p>
      </div>

      <Button
        variant="primary"
        onClick={() => void getCurrentWindow().close()}
        className="px-8"
      >
        关闭
      </Button>
    </div>
  );
}
