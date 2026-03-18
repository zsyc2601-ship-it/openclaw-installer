import { useInstaller } from "./hooks/useInstaller";
import Install from "./pages/Install";
import Progress from "./pages/Progress";
import ApiKey from "./pages/ApiKey";
import Dashboard from "./pages/Dashboard";
import Uninstall from "./pages/Uninstall";

export default function App() {
  const phase = useInstaller((s) => s.phase);

  switch (phase) {
    case "idle":
      return <Install />;
    case "installing":
      return <Progress />;
    case "awaiting_api_key":
      return <ApiKey />;
    case "complete":
      return <Dashboard />;
    case "confirm_uninstall":
      return <Uninstall />;
    case "uninstalling":
      return <Progress />;
    case "uninstalled":
      return <Install />;
    case "error":
      return <Dashboard />;
    default:
      return <Install />;
  }
}
