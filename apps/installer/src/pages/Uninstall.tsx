import { useState } from "react";
import { useInstaller } from "../hooks/useInstaller";

export default function Uninstall() {
  const { startUninstall, error } = useInstaller();
  const [removeData, setRemoveData] = useState(false);

  return (
    <div className="card">
      <h1>卸载 OpenClaw</h1>
      <p>将停止服务、移除 OpenClaw 及内嵌的 Node.js 运行时。</p>

      {error && <div className="error-box">{error}</div>}

      <label className="checkbox-row">
        <input
          type="checkbox"
          checked={removeData}
          onChange={(e) => setRemoveData(e.target.checked)}
        />
        同时删除配置和聊天记录 (~/.openclaw/)
      </label>

      <div className="btn-group">
        <button
          className="btn btn-outline"
          onClick={() => useInstaller.setState({ phase: "complete", error: null })}
        >
          取消
        </button>
        <button
          className="btn btn-danger"
          onClick={() => startUninstall(removeData)}
        >
          确定卸载
        </button>
      </div>
    </div>
  );
}
