import React, { createContext, useReducer, useEffect } from "react";
import { FedimintWallet, OutgoingLightningPayment } from "@fedimint/core-web";
import { WALLET_ACTION_TYPE } from "@/types";
import { wallet } from "@/wallet";
import { LnReceiveState } from "@/types/wallet";

export interface BaseTransaction {
  id: string;
  amount: number;
  timestamp: number;
  invoice: string;
}

export interface SendTransaction extends BaseTransaction {
  type: "send";
  outgoingLightningPayment: OutgoingLightningPayment;
}

export interface ReceiveTransaction extends BaseTransaction {
  type: "receive";
  state: LnReceiveState;
}

export type Transaction = SendTransaction | ReceiveTransaction;

export interface WalletState {
  wallet: FedimintWallet;
  error: string | null;
  transactionHistory: Transaction[];
}

export type WalletAction =
  | { type: WALLET_ACTION_TYPE.SET_ERROR; payload: string | null }
  | { type: WALLET_ACTION_TYPE.ADD_TRANSACTION; payload: Transaction }
  | { type: WALLET_ACTION_TYPE.UPDATE_TRANSACTION; payload: Transaction }
  | { type: WALLET_ACTION_TYPE.INIT; payload: Transaction[] };

export interface WalletContextValue {
  state: WalletState;
  dispatch: React.Dispatch<WalletAction>;
}

const makeInitialState = (): WalletState => {
  return {
    wallet: wallet,
    error: null,
    transactionHistory: [],
  };
};

function walletReducer(state: WalletState, action: WalletAction): WalletState {
  const newState = (() => {
    switch (action.type) {
      case WALLET_ACTION_TYPE.SET_ERROR:
        return { ...state, error: action.payload };
      case WALLET_ACTION_TYPE.ADD_TRANSACTION:
        return {
          ...state,
          transactionHistory: [action.payload, ...state.transactionHistory],
        };
      case WALLET_ACTION_TYPE.UPDATE_TRANSACTION:
        return {
          ...state,
          transactionHistory: state.transactionHistory.map((transaction) =>
            transaction.id === action.payload.id ? action.payload : transaction
          ),
        };
      case WALLET_ACTION_TYPE.INIT:
        return { ...state, transactionHistory: action.payload };
      default:
        return state;
    }
  })();

  // Save only transactionHistory to Chrome storage if it has changed
  if (
    JSON.stringify(newState.transactionHistory) !==
    JSON.stringify(state.transactionHistory)
  ) {
    chrome.storage.local.set({
      transactionHistory: newState.transactionHistory,
    });
    console.log(
      "Transaction history saved to Chrome storage:",
      newState.transactionHistory
    );
  }
  return newState;
}

export const WalletContext = createContext<WalletContextValue>({
  state: makeInitialState(),
  dispatch: () => {},
});

export const WalletProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [state, dispatch] = useReducer(walletReducer, makeInitialState());

  useEffect(() => {
    chrome.storage.local.get(["transactionHistory"], (result) => {
      if (result.transactionHistory) {
        dispatch({
          type: WALLET_ACTION_TYPE.INIT,
          payload: result.transactionHistory,
        });
      }
    });
  }, []);

  if (!state.wallet) {
    return null;
  }

  const contextValue = {
    state,
    dispatch,
  };

  return (
    <WalletContext.Provider value={contextValue}>
      {children}
    </WalletContext.Provider>
  );
};
