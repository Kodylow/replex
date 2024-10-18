import React, { createContext, useReducer } from "react";
import { FedimintWallet } from "@fedimint/core-web";
import { WALLET_ACTION_TYPE } from "@/types";
import { wallet } from "@/wallet";

export interface Transaction {
  id: string;
  type: "send" | "receive";
  amount: number;
  timestamp: number;
  invoice: string;
}

export interface WalletState {
  wallet: FedimintWallet;
  error: string | null;
  transactionHistory: Transaction[];
}

export type WalletAction =
  | { type: WALLET_ACTION_TYPE.SET_ERROR; payload: string | null }
  | { type: WALLET_ACTION_TYPE.ADD_TRANSACTION; payload: Transaction }
  | { type: WALLET_ACTION_TYPE.UPDATE_TRANSACTION; payload: Transaction };

export interface WalletContextValue {
  state: WalletState;
  dispatch: React.Dispatch<WalletAction>;
}

const initialState: WalletState = {
  wallet: wallet,
  error: null,
  transactionHistory: [],
};

function walletReducer(state: WalletState, action: WalletAction): WalletState {
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
    default:
      return state;
  }
}

export const WalletContext = createContext<WalletContextValue>({
  state: initialState,
  dispatch: () => {},
});

export const WalletProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [state, dispatch] = useReducer(walletReducer, initialState);

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
