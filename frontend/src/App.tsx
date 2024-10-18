import "./App.css";
import { WalletProvider } from "./contexts/WalletContext";
import { AppProvider } from "./contexts/AppContext";
import Content from "./Content";
import { ReceiveProvider } from "./contexts/ReceiveContext";
import { SendProvider } from "./contexts/SendContext";

function App() {
  return (
    <AppProvider>
      <WalletProvider>
        <ReceiveProvider>
          <SendProvider>
            <Content />
          </SendProvider>
        </ReceiveProvider>
      </WalletProvider>
    </AppProvider>
  );
}

export default App;
