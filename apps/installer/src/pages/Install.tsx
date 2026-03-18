import { useInstaller } from "../hooks/useInstaller";

export default function Install() {
  const { startInstall, error } = useInstaller();

  return (
    <div className="card">
      <h1>OpenClaw</h1>
      <p>
        AI 助手网关，一键安装即可使用。
        <br />
        无需手动配置环境，全自动完成。
      </p>

      {error && <div className="error-box">{error}</div>}

      <button className="btn btn-primary" onClick={startInstall}>
        一键安装
      </button>

      <p style={{ marginTop: 16, marginBottom: 0, fontSize: 12, opacity: 0.5 }}>
        将自动安装 Node.js 运行时与 OpenClaw 服务
      </p>
    </div>
  );
}
