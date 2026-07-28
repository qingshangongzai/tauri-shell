// Copyright (c) 2026 青山公仔
// 轻壳 (Tauri Shell) is licensed under Mulan PSL v2.
// You can use this software according to the terms and conditions of the Mulan PSL v2.
// You may obtain a copy of Mulan PSL v2 at:
//          http://license.coscl.org.cn/MulanPSL2
// THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND,
// EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT,
// MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
// See the Mulan PSL v2 for more details.

import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { TitleBar } from "@/components/TitleBar";
import { StepNav } from "@/components/StepNav";
import { WelcomePage } from "@/components/pages/WelcomePage";
import { UpdateWelcomePage } from "@/components/pages/UpdateWelcomePage";
import { PathPage } from "@/components/pages/PathPage";
import { OptionsPage } from "@/components/pages/OptionsPage";
import { ProgressPage } from "@/components/pages/ProgressPage";
import { FinishPage } from "@/components/pages/FinishPage";
import { UninstallConfirmPage } from "@/components/pages/UninstallConfirmPage";
import { UninstallProgressPage } from "@/components/pages/UninstallProgressPage";
import { UninstallFinishPage } from "@/components/pages/UninstallFinishPage";
import { CloseAppConfirmDialog } from "@/components/CloseAppConfirmDialog";
import {
  getInstallConfig,
  isAppRunning,
  type InstallConfig,
  type UpdateInfo,
} from "@/lib/ipc";
import { FILE_ASSOC_ENABLED, PRODUCT_NAME } from "@/product";
import {
  INSTALL_STEPS,
  INSTALL_STEP_LABELS,
  UNINSTALL_STEPS,
  UNINSTALL_STEP_LABELS,
  UPDATE_STEPS,
  UPDATE_STEP_LABELS,
  type InstallStep,
  type UninstallStep,
  type UpdateStep,
} from "@/types";

export default function App() {
  const [config, setConfig] = useState<InstallConfig | null>(null);
  const [configError, setConfigError] = useState<string | null>(null);

  useEffect(() => {
    getInstallConfig()
      .then(setConfig)
      .catch((e) => setConfigError(String(e)));
  }, []);

  // 窗口初始不可见，配置就绪渲染出完整页面后再显示，
  // 避免 WebView 启动期间的白框（与主应用同一机制）
  useEffect(() => {
    if (config || configError) void getCurrentWindow().show();
  }, [config, configError]);

  if (configError) {
    return (
      <div className="flex h-screen flex-col bg-bg">
        <TitleBar closable />
        <div className="flex flex-1 items-center justify-center px-12 text-center text-[13px] text-danger">
          读取安装配置失败：{configError}
        </div>
      </div>
    );
  }

  // 配置加载中（本地 invoke 极快，仅一帧空壳）
  if (!config) {
    return (
      <div className="flex h-screen flex-col bg-bg">
        <TitleBar closable />
      </div>
    );
  }

  if (config.mode === "uninstall") return <UninstallWizard config={config} />;
  // updateInfo 作独立 prop 传入，顶层收窄后组件内类型自然非空
  if (config.mode === "update" && config.updateInfo) {
    return <UpdateWizard config={config} info={config.updateInfo} />;
  }
  return <InstallWizard config={config} />;
}

/** 更新向导：三步精简流程，安装选项静默沿用上次安装记录 */
function UpdateWizard({
  config,
  info,
}: {
  config: InstallConfig;
  info: UpdateInfo;
}) {
  const [step, setStep] = useState<UpdateStep>("welcome");
  // 检测到主程序运行中，等待用户确认关闭（未确认不进入更新流程）
  const [confirmClose, setConfirmClose] = useState(false);

  // 更新即以上次选项调现有 start_install（覆盖安装），后端编排零改动
  const options = {
    isSystem: info.isSystem,
    installDir: info.installDir,
    desktopShortcut: info.desktopShortcut,
    fileAssoc: info.fileAssoc,
  };

  // 更新进行中禁用关闭（防止中断写注册表/复制文件）
  const closable = step !== "progress";

  // 进入更新前检测运行中的实例：弹确认对话框，未确认不进入更新流程
  //（实际关闭由后端安装编排执行）
  const confirmThenUpdate = async () => {
    if (await isAppRunning(info.installDir)) {
      setConfirmClose(true);
      return;
    }
    setStep("progress");
  };

  const renderPage = () => {
    switch (step) {
      case "welcome":
        return (
          <UpdateWelcomePage
            info={info}
            version={config.version}
            onNext={() => void confirmThenUpdate()}
          />
        );
      case "progress":
        return (
          <ProgressPage
            options={options}
            isUpdate
            onSuccess={() => setStep("finish")}
            onBack={() => setStep("welcome")}
          />
        );
      case "finish":
        return (
          <FinishPage
            installDir={info.installDir}
            fileAssoc={info.fileAssoc}
            isSystem={info.isSystem}
            isUpdate
          />
        );
    }
  };

  return (
    <div className="flex h-screen flex-col bg-bg">
      <TitleBar closable={closable} />
      <div className="flex min-h-0 flex-1">
        <StepNav
          steps={UPDATE_STEPS}
          labels={UPDATE_STEP_LABELS}
          current={step}
        />
        <main className="min-w-0 flex-1 bg-bg">
          <div key={step} className="wizard-page-enter h-full">
            {renderPage()}
          </div>
        </main>
      </div>
      <CloseAppConfirmDialog
        open={confirmClose}
        confirmLabel="继续更新"
        description={`检测到 ${PRODUCT_NAME} 正在运行，继续更新将关闭它。`}
        onConfirm={() => {
          setConfirmClose(false);
          setStep("progress");
        }}
        onCancel={() => setConfirmClose(false)}
      />
    </div>
  );
}

function InstallWizard({ config }: { config: InstallConfig }) {
  const [step, setStep] = useState<InstallStep>("welcome");
  const [installDir, setInstallDir] = useState(config.systemDefaultDir);
  const [isSystem, setIsSystem] = useState(true);
  const [desktopShortcut, setDesktopShortcut] = useState(true);
  // 登记为"打开方式"属非侵入操作（不接管默认值），功能启用时默认开启
  const [fileAssoc, setFileAssoc] = useState(FILE_ASSOC_ENABLED);
  // 检测到主程序运行中，等待用户确认关闭（未确认不进入安装流程）
  const [confirmClose, setConfirmClose] = useState(false);

  // 切换安装范围：重置为对应默认路径（与第三阶段验证页联动逻辑一致）
  const handleSystemChange = (nextIsSystem: boolean) => {
    setIsSystem(nextIsSystem);
    setInstallDir(
      nextIsSystem ? config.systemDefaultDir : config.userDefaultDir,
    );
  };

  // 目录改到 Program Files 之外时自动取消"为所有用户安装"：
  // 与开关描述一致，也避免自定义用户目录仍以 HKLM + 提权方式安装；
  // 前缀带尾部分隔符，防 "Program Files (x86)" 误判为 Program Files 内；
  // 空输入不触发（用户清空重输不算"换目录"）
  const systemDirPrefix = config.systemDefaultDir
    .slice(0, config.systemDefaultDir.lastIndexOf("\\") + 1)
    .toLowerCase();
  const handleInstallDirChange = (dir: string) => {
    setInstallDir(dir);
    if (
      isSystem &&
      systemDirPrefix &&
      dir.trim() &&
      !dir.toLowerCase().startsWith(systemDirPrefix)
    ) {
      setIsSystem(false);
    }
  };

  // 安装进行中禁用关闭（防止中断写注册表/复制文件）
  const closable = step !== "progress";

  // 进入安装前检测运行中的实例：弹确认对话框，未确认不进入安装流程
  //（实际关闭由后端安装编排执行）
  const confirmThenInstall = async () => {
    if (await isAppRunning(installDir.trim())) {
      setConfirmClose(true);
      return;
    }
    setStep("progress");
  };

  const renderPage = () => {
    switch (step) {
      case "welcome":
        return (
          <WelcomePage
            version={config.version}
            onNext={() => setStep("path")}
          />
        );
      case "path":
        return (
          <PathPage
            installDir={installDir}
            isSystem={isSystem}
            onInstallDirChange={handleInstallDirChange}
            onSystemChange={handleSystemChange}
            onBack={() => setStep("welcome")}
            onNext={() => setStep("options")}
          />
        );
      case "options":
        return (
          <OptionsPage
            desktopShortcut={desktopShortcut}
            fileAssoc={fileAssoc}
            onDesktopShortcutChange={setDesktopShortcut}
            onFileAssocChange={setFileAssoc}
            onBack={() => setStep("path")}
            onNext={() => void confirmThenInstall()}
          />
        );
      case "progress":
        return (
          <ProgressPage
            options={{
              isSystem,
              installDir: installDir.trim(),
              desktopShortcut,
              fileAssoc,
            }}
            onSuccess={() => setStep("finish")}
            onBack={() => setStep("options")}
          />
        );
      case "finish":
        return (
          <FinishPage
            installDir={installDir.trim()}
            fileAssoc={fileAssoc}
            isSystem={isSystem}
          />
        );
    }
  };

  return (
    <div className="flex h-screen flex-col bg-bg">
      <TitleBar closable={closable} />
      <div className="flex min-h-0 flex-1">
        <StepNav
          steps={INSTALL_STEPS}
          labels={INSTALL_STEP_LABELS}
          current={step}
        />
        <main className="min-w-0 flex-1 bg-bg">
          <div key={step} className="wizard-page-enter h-full">
            {renderPage()}
          </div>
        </main>
      </div>
      <CloseAppConfirmDialog
        open={confirmClose}
        confirmLabel="继续安装"
        description={`检测到 ${PRODUCT_NAME} 正在运行，继续安装将关闭它。`}
        onConfirm={() => {
          setConfirmClose(false);
          setStep("progress");
        }}
        onCancel={() => setConfirmClose(false)}
      />
    </div>
  );
}

function UninstallWizard({ config }: { config: InstallConfig }) {
  const [step, setStep] = useState<UninstallStep>("confirm");
  const [removeUserData, setRemoveUserData] = useState(false);
  const [failed, setFailed] = useState(false);
  // 检测到主程序运行中，等待用户确认关闭（未确认不进入卸载流程）
  const [confirmClose, setConfirmClose] = useState(false);

  // 卸载进行中禁用关闭（防止中断删文件/注册表）；失败后恢复可关闭
  const closable = step !== "progress" || failed;

  // 进入卸载前检测运行中的实例：弹确认对话框，未确认不进入卸载流程
  //（实际关闭由后端卸载编排执行）
  const confirmThenUninstall = async () => {
    const dir = config.uninstallInfo?.installDir ?? "";
    if (dir && (await isAppRunning(dir))) {
      setConfirmClose(true);
      return;
    }
    setStep("progress");
  };

  const renderPage = () => {
    switch (step) {
      case "confirm":
        return (
          <UninstallConfirmPage
            info={config.uninstallInfo}
            version={config.version}
            removeUserData={removeUserData}
            onRemoveUserDataChange={setRemoveUserData}
            onUninstall={() => void confirmThenUninstall()}
          />
        );
      case "progress":
        return (
          <UninstallProgressPage
            removeUserData={removeUserData}
            onSuccess={() => setStep("finish")}
            onError={() => setFailed(true)}
          />
        );
      case "finish":
        return <UninstallFinishPage />;
    }
  };

  return (
    <div className="flex h-screen flex-col bg-bg">
      <TitleBar closable={closable} title={`${PRODUCT_NAME} 卸载`} />
      <div className="flex min-h-0 flex-1">
        <StepNav
          steps={UNINSTALL_STEPS}
          labels={UNINSTALL_STEP_LABELS}
          current={step}
        />
        <main className="min-w-0 flex-1 bg-bg">
          <div key={step} className="wizard-page-enter h-full">
            {renderPage()}
          </div>
        </main>
      </div>
      <CloseAppConfirmDialog
        open={confirmClose}
        confirmLabel="继续卸载"
        description={`检测到 ${PRODUCT_NAME} 正在运行，继续卸载将关闭它。`}
        onConfirm={() => {
          setConfirmClose(false);
          setStep("progress");
        }}
        onCancel={() => setConfirmClose(false)}
      />
    </div>
  );
}
