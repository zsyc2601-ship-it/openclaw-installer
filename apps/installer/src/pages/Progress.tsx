import { useInstaller } from "../hooks/useInstaller";

export default function Progress() {
  const { phase, progress } = useInstaller();
  const percent = progress.total > 0 ? (progress.step / progress.total) * 100 : 0;
  const title = phase === "uninstalling" ? "正在卸载..." : "正在安装...";

  return (
    <div className="card">
      <h1>{title}</h1>
      <p>请勿关闭窗口，操作完成后将自动跳转。</p>

      <div className="progress-bar">
        <div className="progress-bar-fill" style={{ width: `${percent}%` }} />
      </div>

      <div className="step-label">
        步骤 {progress.step}/{progress.total}: {progress.label}
      </div>
      <div className="step-detail">{progress.detail}</div>
    </div>
  );
}
