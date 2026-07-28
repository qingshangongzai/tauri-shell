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
import { PRODUCT_NAME, PRODUCT_TAGLINE } from "@/product";

interface WelcomePageProps {
  version: string;
  onNext: () => void;
}

/** 第一步：欢迎页。产品简介 + 版本号 + 开始安装（不含许可协议）。 */
export function WelcomePage({ version, onNext }: WelcomePageProps) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-6 px-12 text-center">
      <img src="/logo.svg" alt={PRODUCT_NAME} className="h-16 w-16" />
      <div className="flex flex-col gap-2">
        <h1 className="text-[22px] font-semibold text-text-primary">
          欢迎使用 {PRODUCT_NAME}
        </h1>
        <p className="text-[13px] text-text-auxiliary">
          {version ? `版本 ${version}` : "\u00a0"}
        </p>
      </div>
      <p className="max-w-md text-[14px] leading-relaxed text-text-body">
        {PRODUCT_TAGLINE}
      </p>
      <Button variant="primary" onClick={onNext} className="mt-2 px-8">
        开始安装
      </Button>
    </div>
  );
}
