import { useCallback, useEffect, useState } from "react";
import { Card, CardContent } from "../components/ui/card";
import BalanceDisplay from "../components/BalanceDisplay";
import ActionButtons from "../components/ActionButtons";
import SendScreen from "./Send";
import ReceiveScreen from "./Receive";
import { Screen } from "../types";
import { useAppCurrentScreen, useAppSetCurrentScreen } from "@/hooks/useApp";
import { useWalletInstance } from "@/hooks/useWallet";
import { Transactions } from "@/components/Transactions";
import { SendProvider } from "@/contexts/SendContext";
import { ReceiveProvider } from "@/contexts/ReceiveContext";

const useIsOpen = () => {
  const wallet = useWalletInstance();
  const [open, setIsOpen] = useState(false);

  const checkIsOpen = useCallback(() => {
    // Check if the wallet object has an isOpen method
    if (wallet && typeof wallet.isOpen === "function") {
      const isOpenNow = wallet.isOpen();
      if (open !== isOpenNow) {
        setIsOpen(isOpenNow);
      }
    } else {
      // If there's no isOpen method, assume it's always open
      setIsOpen(true);
    }
  }, [open, wallet]);

  useEffect(() => {
    checkIsOpen();
  }, [checkIsOpen]);

  return { open, checkIsOpen };
};

const useBalance = (checkIsOpen: () => void) => {
  const wallet = useWalletInstance();
  const [balance, setBalance] = useState(0);

  useEffect(() => {
    if (wallet && typeof wallet.balance.subscribeBalance === "function") {
      const unsubscribe = wallet.balance.subscribeBalance((newBalance) => {
        checkIsOpen();
        setBalance(newBalance);
      });

      return () => {
        if (typeof unsubscribe === "function") {
          unsubscribe();
        }
      };
    }
  }, [checkIsOpen, wallet]);

  return balance;
};

export default function HomeScreen() {
  const currentScreen = useAppCurrentScreen();
  const setCurrentScreen = useAppSetCurrentScreen();
  const { checkIsOpen } = useIsOpen();
  const balance = useBalance(checkIsOpen);

  const renderScreen = () => {
    switch (currentScreen) {
      case Screen.Send:
        return (
          <SendProvider>
            <SendScreen onComplete={() => setCurrentScreen(Screen.Home)} />
          </SendProvider>
        );
      case Screen.Receive:
        return (
          <ReceiveProvider>
            <ReceiveScreen onComplete={() => setCurrentScreen(Screen.Home)} />
          </ReceiveProvider>
        );
      default:
        return (
          <div className="flex flex-col gap-4">
            <BalanceDisplay balance={balance} />
            <ActionButtons
              onSendClick={() => setCurrentScreen(Screen.Send)}
              onReceiveClick={() => setCurrentScreen(Screen.Receive)}
            />
            <Transactions />
          </div>
        );
    }
  };

  return (
    <Card className="w-full border-none">
      <CardContent>{renderScreen()}</CardContent>
    </Card>
  );
}
