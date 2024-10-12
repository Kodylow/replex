import { useAppActiveTab } from "./hooks/useApp";
import { Tab } from "./types";
import Layout from "./components/Layout";
import Header from "./components/Header";
import HomeScreen from "./screens/Home";
import SettingsScreen from "./screens/Settings";
import GrokChat from "./screens/GrokChat";
import Agent from "./screens/Agent";

function Content() {
  const activeTab = useAppActiveTab();

  return (
    <Layout>
      <Header />
      <main className="flex-grow overflow-hidden">
        {activeTab === Tab.Wallet && <HomeScreen />}
        {activeTab === Tab.Settings && <SettingsScreen />}
        {activeTab === Tab.GrokChat && <GrokChat />}
        {activeTab === Tab.Agent && <Agent />}
      </main>
    </Layout>
  );
}

export default Content;