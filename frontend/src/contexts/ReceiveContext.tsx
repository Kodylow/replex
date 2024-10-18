import React, { createContext, useCallback, useState, useContext } from "react";
import { useWalletInstance } from "@/hooks/useWallet";
import { LnReceiveState } from "@/types/wallet";
import { WalletContext, Transaction } from "./WalletContext";
import { WALLET_ACTION_TYPE } from "@/types";

interface ReceiveContextValue {
  invoice: string;
  paid: boolean;
  error: string | null;
  paymentState: LnReceiveState | null;
  createInvoice: (amount: number) => Promise<void>;
  resetState: () => void;
}

export const ReceiveContext = createContext<ReceiveContextValue>({
  invoice: "",
  paid: false,
  error: null,
  paymentState: null,
  createInvoice: async () => {},
  resetState: () => {},
});

export const ReceiveProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const wallet = useWalletInstance();
  const { dispatch } = useContext(WalletContext);
  const [invoice, setInvoice] = useState("");
  const [paid, setPaid] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [paymentState, setPaymentState] = useState<LnReceiveState | null>(null);

  const createInvoice = useCallback(
    async (amount: number) => {
      setError(null);
      setPaymentState(null);
      try {
        const response = await wallet.lightning.createInvoice(
          amount,
          "Receive payment"
        );
        setInvoice(response.invoice);

        const unsubscribe = wallet.lightning.subscribeLnReceive(
          response.operation_id,
          (state: LnReceiveState) => {
            setPaymentState(state);
            if (state === "claimed") {
              setPaid(true);
              unsubscribe();

              // Add transaction to history
              const transaction: Transaction = {
                id: response.operation_id,
                type: "receive",
                amount: amount,
                timestamp: Date.now(),
                invoice: response.invoice,
              };
              dispatch({
                type: WALLET_ACTION_TYPE.ADD_TRANSACTION,
                payload: transaction,
              });
            }
          },
          (error) => {
            setError(error);
            unsubscribe();
          }
        );
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [wallet, dispatch]
  );

  const resetState = useCallback(() => {
    setInvoice("");
    setPaid(false);
    setError(null);
    setPaymentState(null);
  }, []);

  const contextValue = {
    invoice,
    paid,
    error,
    paymentState,
    createInvoice,
    resetState,
  };

  return (
    <ReceiveContext.Provider value={contextValue}>
      {children}
    </ReceiveContext.Provider>
  );
};
