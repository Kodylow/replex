import { useContext, useEffect, useState } from "react";
import { WalletContext, WalletContextValue } from "../contexts/WalletContext";
import { FedimintWallet } from "@fedimint/core-web";

export const useWallet = (): WalletContextValue => {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error("useWallet must be used within a WalletProvider");
  }
  console.log("context", context);
  return context;
};

export const useWalletInstance = (): FedimintWallet => {
  const { state } = useWallet();
  if (!state.wallet) {
    throw new Error("Wallet not initialized");
  }
  return state.wallet;
};

export const useWalletBalance = (): number => {
  const wallet = useWalletInstance();
  const [balance, setBalance] = useState(0);

  useEffect(() => {
    const unsubscribe = wallet.balance.subscribeBalance((balance) => {
      setBalance(balance);
    });

    return () => {
      unsubscribe();
    };
  }, [wallet]);

  return balance;
};
