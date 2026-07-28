// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

interface StepNavProps<S extends string> {
  steps: readonly S[];
  labels: Record<S, string>;
  current: S;
}

/** 左侧步骤导航：仅展示进度，不可点击跳转（安装/卸载两套步骤共用）。 */
export function StepNav<S extends string>({
  steps,
  labels,
  current,
}: StepNavProps<S>) {
  const currentIndex = steps.indexOf(current);

  return (
    <nav className="flex w-32 shrink-0 flex-col gap-2 bg-sidebar px-4 py-6">
      {steps.map((step, index) => {
        const isCurrent = index === currentIndex;
        const isDone = index < currentIndex;
        return (
          <div
            key={step}
            className={cn(
              "flex items-center gap-2 rounded px-2 py-2 text-[13px]",
              isCurrent && "bg-accent-bg font-semibold text-accent",
              isDone && "text-text-body",
              !isCurrent && !isDone && "text-text-placeholder",
            )}
          >
            <span className="flex h-4 w-4 items-center justify-center">
              {isDone ? (
                <Check className="h-4 w-4" strokeWidth={2} />
              ) : (
                <span className="text-[12px] tabular-nums">{index + 1}</span>
              )}
            </span>
            {labels[step]}
          </div>
        );
      })}
    </nav>
  );
}
