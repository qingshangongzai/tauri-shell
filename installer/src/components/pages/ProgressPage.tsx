// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { useEffect, useRef, useState } from "react";
import { AlertCircle } from "lucide-react";
import { Button } from "@/components/ui/Button";
import {
  startInstall,
  onInstallProgress,
  type InstallOptions,
  type InstallProgress,
} from "@/lib/ipc";

interface ProgressPageProps {
  options: InstallOptions;
  onSuccess: () => void;
  onBack: () => void;
}

/** 第四步：安装进度。进入即触发 start_install，事件驱动进度条。 */
export function ProgressPage({
  options,
  onSuccess,
  onBack,
}: ProgressPageProps) {
  const [percent, setPercent] = useState(0);
  const [message, setMessage] = useState("准备安装...");
  const [error, setError] = useState<string | null>(null);
  // StrictMode 会二次触发 effect；install 有副作用（写文件/注册表），必须仅执行一次
  const startedRef = useRef(false);

  useEffect(() => {
    if (startedRef.current) return;
    startedRef.current = true;

    let unlisten: (() => void) | undefined;
    let cancelled = false;

    onInstallProgress((p: InstallProgress) => {
      if (cancelled) return;
      if (p.error) {
        setError(p.error);
      } else {
        setPercent(p.percent);
        setMessage(p.message);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    startInstall(options)
      .then(() => {
        if (cancelled) return;
        setPercent(100);
        onSuccess();
      })
      .catch((e) => {
        if (cancelled) return;
        setError(String(e));
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
    // options 在安装期间固定不变，仅首次挂载执行
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex h-full flex-col gap-8 px-12 py-10">
      <div className="flex flex-col gap-1">
        <h2 className="text-[18px] font-semibold text-text-primary">
          {error ? "安装失败" : "正在安装"}
        </h2>
        <p className="text-[13px] text-text-auxiliary">
          {error
            ? "安装过程中遇到问题。"
            : "请稍候，正在将 Air Note 安装到您的电脑。"}
        </p>
      </div>

      {error ? (
        <div className="flex flex-1 flex-col gap-4">
          <div className="flex items-start gap-2 rounded bg-danger-bg px-4 py-3 text-[13px] text-danger">
            <AlertCircle
              className="mt-0.5 h-4 w-4 shrink-0"
              strokeWidth={1.75}
            />
            <span className="whitespace-pre-wrap">{error}</span>
          </div>
        </div>
      ) : (
        <div className="flex flex-1 flex-col gap-3">
          <div className="h-2 w-full overflow-hidden rounded-full bg-divider">
            <div
              className="h-full rounded-full bg-accent transition-[width] duration-[var(--duration-normal)] ease-[var(--ease-standard)]"
              style={{ width: `${percent}%` }}
            />
          </div>
          <p className="text-[12px] text-text-auxiliary">
            {message} ({percent}%)
          </p>
        </div>
      )}

      {error && (
        <div className="mt-auto flex justify-start">
          <Button variant="default" onClick={onBack}>
            返回上一步
          </Button>
        </div>
      )}
    </div>
  );
}
