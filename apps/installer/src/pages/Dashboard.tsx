import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useInstaller } from "../hooks/useInstaller";

export default function Dashboard() {
  const { phase, error, gatewayUrl } = useInstaller();
  const [healthy, setHealthy] = useState<boolean | null>(null);

  useEffect(() => {
    const check = async () => {
      try {
        const ok = await invoke<boolean>("check_health");
        setHealthy(ok);
      } catch {
        setHealthy(false);
      }
    };
    check();
    const timer = setInterval(check, 5000);
    return () => clearInterval(timer);
  }, []);

  const openConsole = async () => {
    try {
      await invoke("open_url", { url: gatewayUrl });
    } catch {
      window.open(gatewayUrl, "_blank");
    }
  };

  return (
    <div className="card">
      <h1>{phase === "error" ? "出现错误" : "安装完成"}</h1>

      {error && <div className="error-box">{error}</div>}

      <div style={{ marginBottom: 24 }}>
        <div className="status-row">
          <span>
            <span className={`status-dot ${healthy ? "green" : "red"}`} />
            Gateway 状态
          </span>
          <span>{healthy === null ? "检测中..." : healthy ? "运行中" : "未运行"}</span>
        </div>
        <div className="status-row">
          <span>地址</span>
          <span style={{ fontFamily: "monospace", fontSize: 13 }}>{gatewayUrl}</span>
        </div>
      </div>

      <div className="btn-group">
        <button className="btn btn-primary" onClick={openConsole}>
          打开控制台
        </button>
        <button
          className="btn btn-danger"
          onClick={() => useInstaller.setState({ phase: "confirm_uninstall" })}
        >
          卸载
        </button>
      </div>
    </div>
  );
}
