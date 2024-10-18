import React, { createContext, useCallback, useContext, useState } from "react";
import { WalletContext, Transaction } from "./WalletContext";
import { WALLET_ACTION_TYPE } from "@/types";

interface SendContextValue {
  sending: boolean;
  error: string | null;
  payInvoice: (invoice: string) => Promise<void>;
  resetState: () => void;
}

export const SendContext = createContext<SendContextValue>({
  sending: false,
  error: null,
  payInvoice: async () => {},
  resetState: () => {},
});

export const SendProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const {
    state: { wallet },
    dispatch,
  } = useContext(WalletContext);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const payInvoice = useCallback(
    async (invoice: string) => {
      setError(null);
      setSending(true);
      try {
        const result = await wallet.lightning.payInvoice(invoice);

        // Add transaction to history
        const transaction: Transaction = {
          id: result.contract_id,
          type: "send",
          // TODO: Get amount from invoice with bolt11 lib
          amount: 0,
          timestamp: Date.now(),
          invoice: invoice,
        };
        dispatch({
          type: WALLET_ACTION_TYPE.ADD_TRANSACTION,
          payload: transaction,
        });
      } catch (e) {
        console.error("Error sending Lightning payment", e);
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setSending(false);
      }
    },
    [wallet, dispatch]
  );

  const resetState = useCallback(() => {
    setSending(false);
    setError(null);
  }, []);

  const contextValue = {
    sending,
    error,
    payInvoice,
    resetState,
  };

  return (
    <SendContext.Provider value={contextValue}>{children}</SendContext.Provider>
  );
};
