import { useState } from "react";
import { useInstaller } from "../hooks/useInstaller";

const PROVIDERS = [
  { value: "claude", label: "Claude (Anthropic)" },
  { value: "openai", label: "OpenAI" },
  { value: "gemini", label: "Gemini (Google)" },
];

export default function ApiKey() {
  const { submitApiKey, error } = useInstaller();
  const [provider, setProvider] = useState("claude");
  const [key, setKey] = useState("");
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async () => {
    if (!key.trim()) return;
    setSubmitting(true);
    await submitApiKey(provider, key.trim());
    setSubmitting(false);
  };

  return (
    <div className="card">
      <h1>配置 API Key</h1>
      <p>
        OpenClaw 需要至少一个 AI 服务的 API Key 才能工作。
        <br />
        你可以稍后在控制台中修改。
      </p>

      {error && <div className="error-box">{error}</div>}

      <div className="form-group">
        <label>AI 服务商</label>
        <select value={provider} onChange={(e) => setProvider(e.target.value)}>
          {PROVIDERS.map((p) => (
            <option key={p.value} value={p.value}>
              {p.label}
            </option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label>API Key</label>
        <input
          type="password"
          placeholder="sk-ant-..."
          value={key}
          onChange={(e) => setKey(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSubmit()}
        />
      </div>

      <button
        className="btn btn-primary"
        onClick={handleSubmit}
        disabled={!key.trim() || submitting}
      >
        {submitting ? "保存中..." : "保存并完成"}
      </button>
    </div>
  );
}
