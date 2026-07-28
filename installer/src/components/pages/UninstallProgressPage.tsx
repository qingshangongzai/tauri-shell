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
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Button } from "@/components/ui/Button";
import {
  startUninstall,
  onInstallProgress,
  type InstallProgress,
} from "@/lib/ipc";
import { PRODUCT_NAME } from "@/product";

interface UninstallProgressPageProps {
  removeUserData: boolean;
  onSuccess: () => void;
  /** 失败时通知向导恢复标题栏关闭按钮（step 停留在 progress） */
  onError: () => void;
}

/** 卸载第二步：进度页。进入即触发 start_uninstall，事件驱动进度条。 */
export function UninstallProgressPage({
  removeUserData,
  onSuccess,
  onError,
}: UninstallProgressPageProps) {
  const [percent, setPercent] = useState(0);
  const [message, setMessage] = useState("准备卸载...");
  const [error, setError] = useState<string | null>(null);
  // StrictMode 会二次触发 effect；卸载有副作用（删文件/注册表），必须仅执行一次
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
        onError();
      } else {
        setPercent(p.percent);
        setMessage(p.message);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    startUninstall({ removeUserData })
      .then(() => {
        if (cancelled) return;
        setPercent(100);
        onSuccess();
      })
      .catch((e) => {
        if (cancelled) return;
        setError(String(e));
        onError();
      });

    return () => {
      cancelled = true;
      unlisten?.();
    };
    // removeUserData/回调在卸载期间固定不变，仅首次挂载执行
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="flex h-full flex-col gap-8 px-12 py-10">
      <div className="flex flex-col gap-1">
        <h2 className="text-[18px] font-semibold text-text-primary">
          {error ? "卸载失败" : "正在卸载"}
        </h2>
        <p className="text-[13px] text-text-auxiliary">
          {error
            ? "卸载过程中遇到问题。"
            : `请稍候，正在从您的电脑移除 ${PRODUCT_NAME}。`}
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
          <Button
            variant="default"
            onClick={() => void getCurrentWindow().close()}
          >
            关闭
          </Button>
        </div>
      )}
    </div>
  );
}
